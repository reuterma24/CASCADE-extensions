using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.CSharp;
using static Utils;

public static class CodeExtractor
{
    private const string CSHARP = "csharp";
    private static uint id = 0;

    public static int errorCounterNull = 0;
    public static int errorCounterClass = 0;
    public static int errorCounterExtraction = 0;

    public static async Task AnalyzeFile(Document file, Dictionary<string, Context> contextMap)
    {
        var semanticModel = await file.GetSemanticModelAsync();
        var tree = await file.GetSyntaxTreeAsync();
        if (tree == null || semanticModel == null)
        {
            errorCounterNull++;
            return;
        }

        var root = (CompilationUnitSyntax)await tree.GetRootAsync();
        if (!ContainsClasses(root))
        {
            errorCounterClass++;
            return;
        }

        var methods = root.DescendantNodes().OfType<MethodDeclarationSyntax>()
            .Where(m => m != null && !IsMethodPartOfStruct(m) && !IsMethodPartOfInterface(m))
            .ToArray();

        foreach (var method in methods)
        {
            ProcessMethodAsync(file, contextMap, root, semanticModel, method);
        }
    }

    private static void ProcessMethodAsync(Document file, Dictionary<string, Context> contextMap, CompilationUnitSyntax root, SemanticModel semanticModel, MethodDeclarationSyntax method)
    {
        try
        {
            var semanticMethodInfo = semanticModel.GetDeclaredSymbol(method);
            string key = GetMethodKey(semanticMethodInfo!);
            Context context = ExtractMethodContext(file, root, method);

            contextMap.Add(key, context);
        }
        catch (Exception e)
        {
            errorCounterExtraction++;
            Console.WriteLine($"Error for method '{method.Identifier}' in file '{file.FilePath}'!");
            Console.WriteLine($"Exception:\n {e}\n");
        }
    }

    private static Context ExtractMethodContext(Document document, CompilationUnitSyntax root, MethodDeclarationSyntax method)
    {
        string rootPath = Path.GetDirectoryName(document.Project.Solution.FilePath) ?? string.Empty;
        string relativeFilePath = Path.GetRelativePath(rootPath, document.FilePath ?? string.Empty);

        return new Context()
        {
            Id = id++,
            RootPath = rootPath,
            Doc = GetDocumentation(method),
            Signature = GetMethodSignature(method),
            Language = CSHARP,
            Parent = GetMethodParent(method, root),
            Code = method.Body?.ToString() ?? string.Empty,
            CodeFilePath = relativeFilePath,
            CalledFunctions = GetCalledFunctions(method)
        };
    }

    private static Parent GetMethodParent(MethodDeclarationSyntax method, CompilationUnitSyntax root)
    {
        var classDeclaration = root.DescendantNodes().OfType<ClassDeclarationSyntax>()
            .First(c => ContainsMethod(c, method));

        var methods = classDeclaration.Members.OfType<MethodDeclarationSyntax>();
        OtherMethod[] otherMethods = methods
            .Where(m => m != method)
            .Select(m =>
                new OtherMethod
                {
                    Doc = GetDocumentation(m),
                    Signature = GetMethodSignature(m),
                    Code = m.Body?.ToString() ?? string.Empty
                }
            )
            .ToArray();

        var variables = classDeclaration.Members.OfType<FieldDeclarationSyntax>();
        var variableNames = variables
            .Select(f => f.NormalizeWhitespace().ToFullString())
            .ToArray();

        var generics = classDeclaration.TypeParameterList?.Parameters
            .Select(param => param.Identifier.Text)
            .ToArray() ?? [];

        var constructors = classDeclaration.Members.OfType<ConstructorDeclarationSyntax>()
            .Select(c => c.NormalizeWhitespace().ToFullString())
            .ToArray();

        // Extract the base list (which contains both the base class and interfaces)
        var baseList = classDeclaration.BaseList?.Types
            .Select(t => t.ToString())
            .ToArray() ?? [];

        var extends = baseList.FirstOrDefault() ?? string.Empty;

        var implements = baseList.Skip(1).ToArray();

        var namespaceName = GetNamespace(root);

        return new Parent()
        {
            Name = classDeclaration.Identifier.Text,
            Doc = GetDocumentation(classDeclaration),
            OtherMethods = otherMethods,
            Variables = variableNames,
            Generics = generics,
            Imports = root.Usings.Select(u => u.ToString()).ToArray(),
            Extends = extends,
            Implements = implements,
            Constructors = constructors,
            Namespace = namespaceName
        };
    }

    public static Signature GetMethodSignature(MethodDeclarationSyntax method)
    {
        return new Signature()
        {
            Name = method.Identifier.Text,
            Returns = method.ReturnType.ToString(),
            Params = method.ParameterList.Parameters.Select(p => p.ToString()).ToArray(),
            Modifier = method.Modifiers.Select(m => m.Text).ToArray(),
            Annotations = method.AttributeLists.SelectMany(attrList => attrList.Attributes).Select(attr => attr.Name.ToString()).ToArray(),
            Generics = method.TypeParameterList?.Parameters.Select(param => param.Identifier.Text).ToArray() ?? []
        };
    }

    static string GetDocumentation(SyntaxNode node)
    {
        var trivia = node.GetLeadingTrivia();
        if (trivia.Any())
            trivia = trivia.NormalizeWhitespace();

        var xmlComments = trivia
            .Where(t => t.IsKind(SyntaxKind.SingleLineDocumentationCommentTrivia) || t.IsKind(SyntaxKind.MultiLineDocumentationCommentTrivia));
        var comments = trivia
            .Where(t => t.IsKind(SyntaxKind.SingleLineCommentTrivia) || t.IsKind(SyntaxKind.MultiLineCommentTrivia));

        var doc = xmlComments.Concat(comments)
            .OrderBy(c => c.SpanStart)
            .Select(c => c.ToFullString().Trim());

        return string.Join(Environment.NewLine, doc);
    }

    public static string[] GetCalledFunctions(MethodDeclarationSyntax method)
    {
        var calledFunctions = method.Body?.DescendantNodes()
            .OfType<InvocationExpressionSyntax>()
            .Select(invocation => invocation.ToString())
            .ToArray() ?? [];

        return calledFunctions;
    }
}