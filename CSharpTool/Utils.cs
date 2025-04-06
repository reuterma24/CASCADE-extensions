using Microsoft.CodeAnalysis.CSharp.Syntax;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Xml.Linq;
using Microsoft.CodeAnalysis;

public static class Utils
{
    const string xUnit = "xUnit";
    const string NUnit = "NUnit";
    const string MSTest = "MSTest";

    private static readonly string[] TestFrameworks = [
        "nunit",
        "mstest",
        "xunit"
    ];

    private static readonly string[] TestAnnotations = [
        "TestFixture",      // NUnit
        "Test",             // NUnit
        "TestCase",         // NUnit
        "TestClass",        // MSTest
        "TestMethod",       // MSTest
        "DataTestMethod",   // MSTest
        "Fact",             // xUnit
        "Theory"            // xUnit
    ];

    private static readonly string[] DirectoriesToIgnore = [
       "bin", "obj"
   ];

    public static bool ContainsMethod(ClassDeclarationSyntax c, MethodDeclarationSyntax method)
    {
        return c.Members.Contains(method);
    }

    public static bool ContainsClasses(CompilationUnitSyntax root)
    {
        return root.DescendantNodes().OfType<ClassDeclarationSyntax>().Any();
    }

    public static bool IsMethodPartOfStruct(MethodDeclarationSyntax method)
    {
        return method.Ancestors().OfType<StructDeclarationSyntax>().Any();
    }

    public static bool IsMethodPartOfInterface(MethodDeclarationSyntax method)
    {
        return method.Ancestors().OfType<InterfaceDeclarationSyntax>().Any();
    }

    public static bool IsTestProject(Project project)
    {
        // Parse the .csproj file as XML and check for the <IsTestProject>true</IsTestProject> flag
        var projectFilePath = project.FilePath;
        if (File.Exists(projectFilePath))
        {
            var projectXml = XDocument.Load(projectFilePath);
            var isTestProject = projectXml.Descendants("IsTestProject")
                .Any(e => bool.TryParse(e.Value, out bool result) && result);

            if (isTestProject)
            {
                return true;
            }
        }

        // Check project references for popular test frameworks
        if (project.MetadataReferences
            .Where(m => m.Display != null)
            .Any(m => TestFrameworks.Contains(m.Display!.ToLower())))
        {
            return true;
        }

        // Check if the project name contains "Test"
        if (project.Name.Contains("Test", StringComparison.OrdinalIgnoreCase))
        {
            return true;
        }

        return false;
    }

    public static bool ContainsTestClass(CompilationUnitSyntax root)
    {
        // Find all class declarations in the file
        var classDeclarations = root.DescendantNodes().OfType<ClassDeclarationSyntax>();

        return classDeclarations.Any(IsTestClass);
    }

    public static bool IsTestClass(ClassDeclarationSyntax classDeclaration)
    {
        // Check for attributes that indicate a test class
        var hasTestClassAttributes = classDeclaration.AttributeLists
            .SelectMany(al => al.Attributes)
            .Any(attr => IsTestAttribute(attr.Name.ToString()));

        if (hasTestClassAttributes)
        {
            return true;
        }

        // Check for methods that are annotated with test attributes
        var methodsInClass = classDeclaration.DescendantNodes().OfType<MethodDeclarationSyntax>();
        bool hasTestMethodAttributes = methodsInClass.Any(IsTestMethod);

        return hasTestMethodAttributes;
    }

    public static bool IsTestMethod(MethodDeclarationSyntax method)
    {
        return method.AttributeLists
            .SelectMany(a => a.Attributes)
            .Any(attr => IsTestAttribute(attr.Name.ToString()));
    }

    public static bool IsTestAttribute(string attributeName)
    {
        return TestAnnotations.Contains(attributeName);
    }

    public static bool IsDeclaredInSolution(IMethodSymbol method)
    {
        return method.ContainingType.DeclaringSyntaxReferences.Length > 0;
    }

    public static bool IsCSharpFile(Document document)
    {
        return document.FilePath?.EndsWith(".cs") ?? false;
    }

    public static bool IsValidDocument(Document document)
    {
        if (IsCSharpFile(document))
        {
            return !DirectoriesToIgnore.Any(document.FilePath!.Contains);
        }
        else
        {
            return false;
        }
    }

    public static string GetMethodKey(IMethodSymbol methodSymbol)
    {
        ArgumentNullException.ThrowIfNull(methodSymbol);

        var keyBuilder = new StringBuilder();

        keyBuilder.Append(methodSymbol.ContainingType.ConstructedFrom.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat));

        keyBuilder.Append(methodSymbol.Name);

        keyBuilder.Append(methodSymbol.ReturnType.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat));

        // Iterate over parameters, skipping the first if it's an extension method
        bool isExtensionMethod = methodSymbol.IsExtensionMethod;
        var parametersToConsider = isExtensionMethod ? methodSymbol.Parameters.Skip(1) : methodSymbol.Parameters;

        foreach (var parameter in parametersToConsider)
        {
            keyBuilder.Append(parameter.Type.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat));
        }

        return keyBuilder.ToString();
    }

    // Method to detect test framework based on attributes or using directives
    public static string DetectTestFramework(CompilationUnitSyntax root)
    {
        // Check using directives for test framework-specific namespaces
        var usingDirectives = root.Usings.Select(u => u.Name?.ToString() ?? string.Empty);

        if (usingDirectives.Any(u => u.Contains("Xunit")))
            return xUnit;
        if (usingDirectives.Any(u => u.Contains("NUnit")))
            return NUnit;
        if (usingDirectives.Any(u => u.Contains("Microsoft.VisualStudio.TestTools.UnitTesting")))
            return MSTest;

        // Check method or class attributes for test framework-specific attributes
        var testMethodAttributes = root.DescendantNodes()
            .OfType<MethodDeclarationSyntax>()
            .SelectMany(method => method.AttributeLists.SelectMany(a => a.Attributes));

        if (testMethodAttributes.Any(a => a.Name.ToString().Contains("Fact") || a.Name.ToString().Contains("Theory")))
            return xUnit;
        if (testMethodAttributes.Any(a => a.Name.ToString().Contains("Test") || a.Name.ToString().Contains("TestCase")))
            return NUnit;
        if (testMethodAttributes.Any(a => a.Name.ToString().Contains("TestMethod") || a.Name.ToString().Contains("DataTestMethod")))
            return MSTest;

        return string.Empty;
    }

    public static string GetNamespace(CompilationUnitSyntax root)
    {
        var namespaceDeclaration = root.DescendantNodes()
            .OfType<NamespaceDeclarationSyntax>()
            .FirstOrDefault();

        if (namespaceDeclaration != null)
            return namespaceDeclaration.Name.ToString();

        // Handle file-scoped namespace (C# 10+)
        var fileScopedNamespace = root.DescendantNodes()
            .OfType<FileScopedNamespaceDeclarationSyntax>()
            .FirstOrDefault();

        if (fileScopedNamespace != null)
            return fileScopedNamespace.Name.ToString();

        return string.Empty;
    }

    public static void PrintContext(Context context)
    {
        var options = new JsonSerializerOptions { WriteIndented = true, Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping };
        string jsonString = JsonSerializer.Serialize(context, options);
        Console.WriteLine(jsonString);
        Console.WriteLine(", ");
    }
}