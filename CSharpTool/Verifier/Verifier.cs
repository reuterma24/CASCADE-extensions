using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;

public class Verifier
{
    public static int Verify(string[] args)
    {
        if (args.Length < 1)
        {
            Console.WriteLine("Usage: cSharpTool verify <cSharpFile>");
            return -1;
        }

        string filePath = args[0];

        if (!File.Exists(filePath))
        {
            Console.WriteLine($"Error: File not found at {filePath}");
            return -1;
        }

        string code = File.ReadAllText(filePath);

        SyntaxTree syntaxTree = CSharpSyntaxTree.ParseText(code);
        var diagnostics = syntaxTree.GetDiagnostics();
        bool syntacticallyCorrect = !diagnostics.Any(d => d.Severity == DiagnosticSeverity.Error);

        if (syntacticallyCorrect)
        {
            Console.WriteLine("Syntactically correct");
            return 0;
        }
        else
        {
            PrintErrors(diagnostics);
            return -1;
        }
    }

    private static void PrintErrors(IEnumerable<Diagnostic> diagnostics)
    {
        var errors = diagnostics.Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
        foreach (var e in errors)
        {
            var lineSpan = e.Location.GetLineSpan();
            var lineNumber = lineSpan.StartLinePosition.Line + 1;
            var columnNumber = lineSpan.StartLinePosition.Character + 1;

            Console.WriteLine($"Error: {e.GetMessage()}");
            Console.WriteLine($"Line: {lineNumber}, Column: {columnNumber}");
            Console.WriteLine();
        }
    }
}