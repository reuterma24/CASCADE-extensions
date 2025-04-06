using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using System.Text.Json;
using static CodeExtractor;
using static Utils;

public class Modifier
{
    const string NEW_TEST = "new_tests";
    const string NEW_CODE = "new_code";

    public static void Modify(string[] args)
    {
#if DEBUG
        string projectDir = "/Users/mar/Desktop/Masterarbeit/tmp_out/Bank";
        string contextFilePath = "/Users/mar/Desktop/Masterarbeit/tmp_out/entry.json";
        string codeKeyword = "new_code";
        string testKeyword = "new_tests";
#else
        string projectDir = args[0];
        string contextFilePath = args[1];
        string codeKeyword = args[2];
        string testKeyword = args[3];
#endif

        using var filestream = File.OpenRead(contextFilePath);
        Context context = JsonSerializer.Deserialize<Context>(filestream)
            ?? throw new InvalidOperationException($"Could not deserialize {contextFilePath}");

        if (codeKeyword == NEW_CODE)
        {
            ReplaceMethod(projectDir, context);
        }

        if (testKeyword == NEW_TEST)
        {
            AddGeneratedTestFile(context, projectDir);
        }

        Console.WriteLine($"Modification successful! - {testKeyword} - {codeKeyword}");
    }

    private static void ReplaceMethod(string projectDir, Context context)
    {
        if (string.IsNullOrEmpty(context.NewCode))
        {
            throw new InvalidOperationException($"The new methody body (new_code property) is null or empty! Verify the passed JSON file.");
        }

        ReplaceMethodBody(context, context.NewCode, projectDir);
    }

    public static void ReplaceMethodBody(Context context, string methodBody, string projectDir)
    {
        var filePath = Path.Combine(projectDir, context.CodeFilePath);
        var fileContent = File.ReadAllText(filePath);

        var syntaxTree = CSharpSyntaxTree.ParseText(fileContent);
        var root = (CompilationUnitSyntax)syntaxTree.GetRoot();

        var classNode = root.DescendantNodes().OfType<ClassDeclarationSyntax>()
            .FirstOrDefault(cds => cds.Identifier.Text == context.Parent.Name)
            ?? throw new InvalidOperationException("File does not contain class of method!");

        var methodDeclaration = classNode.DescendantNodes().OfType<MethodDeclarationSyntax>()
            .FirstOrDefault(mds => GetMethodSignature(mds) == context.Signature)
            ?? throw new InvalidOperationException("Method not found inside class definition!");

        var currentMethodBody = methodDeclaration.DescendantNodes().OfType<BlockSyntax>().First();

        // insert new method body
        var newMethodSyntaxTree = CSharpSyntaxTree.ParseText(methodBody);
        var newMethodBody = newMethodSyntaxTree.GetRoot().DescendantNodes().OfType<BlockSyntax>().FirstOrDefault()
            ?? throw new InvalidOperationException("Provided method body is not a valid block!");

        var newRoot = root.ReplaceNode(currentMethodBody, newMethodBody);

        File.WriteAllText(filePath, newRoot.NormalizeWhitespace().ToFullString());
    }

    private static void AddGeneratedTestFile(Context context, string projectDir)
    {
        if (string.IsNullOrEmpty(context.NewTests))
        {
            throw new InvalidOperationException($"The new test class (new_tests property) is null or empty! Verify the passed JSON file.");
        }

        //TODO: decide how to handle multiple tests...
        Test test = context.Tests.First();

        string filePath = Path.Combine(projectDir, test.TestFilePath);
        File.WriteAllText(filePath, context.NewTests);
    }

    [Obsolete("We currently rather replace the full file instead of appending test methods to an existing test suite.")]
    public static void ReplaceTestMethods(string filePath, string generatedTests)
    {
        var syntaxTree = CSharpSyntaxTree.ParseText(File.ReadAllText(filePath));
        var root = (CompilationUnitSyntax)syntaxTree.GetRoot();

        var testClass = root.DescendantNodes().OfType<ClassDeclarationSyntax>().FirstOrDefault(IsTestClass)
             ?? throw new InvalidOperationException("File does not contain a test class!");

        var generatedTestClass = GenerateTestsSuite(generatedTests);
        var generatedTestSyntaxTree = CSharpSyntaxTree.ParseText(generatedTestClass);
        var generatedMethods = generatedTestSyntaxTree.GetRoot().DescendantNodes().OfType<MethodDeclarationSyntax>();

        // ... keep static variables or helper methods
        var classMembersToRetain = testClass.Members
            .Where(m => m is not MethodDeclarationSyntax || !IsTestMethod((MethodDeclarationSyntax)m))
            .ToArray();

        // Replace existing test methods with the new ones
        var content = classMembersToRetain.Concat(generatedMethods);
        var updatedClass = testClass.WithMembers(SyntaxFactory.List(content));
        var newRoot = root.ReplaceNode(testClass, updatedClass);

        File.WriteAllText(filePath, newRoot.NormalizeWhitespace().ToFullString());
    }

    private static string GenerateTestsSuite(string testMethods) => $"public class Suite {{ {testMethods} }}";
}