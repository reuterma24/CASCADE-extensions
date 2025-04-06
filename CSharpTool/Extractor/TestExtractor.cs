using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using static Utils;

public static class TestExtractor
{
    public static async Task AppendTestInformation(Document file, Dictionary<string, Context> contextMap)
    {
        var semanticModel = await file.GetSemanticModelAsync();
        var tree = await file.GetSyntaxTreeAsync();
        if (tree is null || semanticModel is null)
            return;

        var root = (CompilationUnitSyntax)await tree.GetRootAsync();
        if (!ContainsTestClass(root))
            return;

        var testMethods = root.DescendantNodes()
            .OfType<MethodDeclarationSyntax>()
            .Where(IsTestMethod);

        var invocations = testMethods
            .Where(tm => tm.Body != null)
            .SelectMany(tm => GetMethodInvocations(tm.Body!)).ToArray();

        var methodsInTestMethods = invocations
            .Select(i => GetMethodSymbolFromInvocation(i, semanticModel))
            .Where(ms => ms != null && IsDeclaredInSolution(ms))
            .ToArray();

        foreach (var call in methodsInTestMethods)
        {
            string key = GetMethodKey(call!.OriginalDefinition.ConstructedFrom);

            // see if method exists in map and add test
            if (contextMap.TryGetValue(key, out var currentContext))
            {
                await AddTest(file, contextMap, root, key, currentContext);
            }
        }
    }

    private static async Task AddTest(Document file, Dictionary<string, Context> contextMap, CompilationUnitSyntax root, string key, Context currentContext)
    {
        string relativeTestFilePath = Path.GetRelativePath(currentContext.RootPath, file.FilePath ?? string.Empty);

        // don't add a file as test source multiple times
        if (currentContext.Tests.Select(t => t.TestFilePath).Contains(relativeTestFilePath))
            return;

        var testInformation = await BuildTestInformation(file, root, relativeTestFilePath);
        contextMap[key].Tests.Add(testInformation);
    }

    private static async Task<Test> BuildTestInformation(Document file, CompilationUnitSyntax root, string relativeTestFilePath)
    {
        string rootPath = Path.GetDirectoryName(file.Project.Solution.FilePath) ?? string.Empty;
        var sourceCode = await file.GetTextAsync();
        var usingDirectives = GetUsingDirectives(root);
        var testRunner = DetectTestFramework(root);
        string namespaceName = GetNamespace(root);
        string testClassName = GetTestClassName(root);
        string relativeTestProjectPath = Path.GetRelativePath(rootPath, file.Project.FilePath ?? string.Empty);

        Test test = new()
        {
            Tests = sourceCode.ToString(),
            TestImports = usingDirectives,
            TestFilePath = relativeTestFilePath,
            TestRunner = testRunner,
            TestNamespace = namespaceName,
            TestClassName = testClassName,
            ProjectPath = relativeTestProjectPath
        };

        return test;
    }

    private static string GetTestClassName(CompilationUnitSyntax root)
    {
        var classDeclaration = root.DescendantNodes()
            .OfType<ClassDeclarationSyntax>()
            .First();

        return classDeclaration.Identifier.Text;
    }

    private static string[] GetUsingDirectives(CompilationUnitSyntax root)
    {
        var usingDirectives = root.DescendantNodes().OfType<UsingDirectiveSyntax>();

        return usingDirectives?
            .Select(u => u.ToString())
            .Distinct()
            .ToArray() ?? [];
    }

    private static IEnumerable<InvocationExpressionSyntax> GetMethodInvocations(BlockSyntax block)
    {
        if (block == null) return [];

        var invocations = block.DescendantNodes()
            .OfType<InvocationExpressionSyntax>()
            .Distinct();

        return invocations;
    }

    private static IMethodSymbol? GetMethodSymbolFromInvocation(InvocationExpressionSyntax invocation, SemanticModel semanticModel)
    {
        var symbolInfo = semanticModel.GetSymbolInfo(invocation);
        if (symbolInfo.Symbol == null && symbolInfo.CandidateSymbols.Any())
        {
            return (IMethodSymbol?)symbolInfo.CandidateSymbols.First();
        }
        return (IMethodSymbol?)symbolInfo.Symbol;
    }
}