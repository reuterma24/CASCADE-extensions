using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.MSBuild;
using Microsoft.Build.Locator;
using System.Text.Json;
using System.Text.Encodings.Web;
using static Utils;

// TODO: remove comments from tests?

public class Extractor
{
    public static async Task Extract(string[] args)
    {
#if DEBUG
        string solutionPath = "/Users/mar/Desktop/Masterarbeit/dummy_project/Bank/Bank.sln";
        string outputPath = "/Users/mar/Desktop/Masterarbeit";
        string targetFramework = "net6.0";
#else
        if (args.Length < 3)
        {
            Console.WriteLine("Usage: cSharpTool extract <inputPath> <outputPath> <targetFramework> (like 'net6.0', 'net8.0')");
            return;
        }

        string solutionPath = args[0];
        string outputPath = args[1];
        string targetFramework = args[2];
#endif

        var extractedContexts = await CreateContexts(solutionPath, targetFramework);
        var contextsWithTests = Filter(extractedContexts);

        Console.WriteLine($"Total of {extractedContexts.Length} methods inside solution.");
        Console.WriteLine($"Matched {contextsWithTests.Length} method with their tests.");
        Console.WriteLine($"Encountered {extractedContexts.Length - contextsWithTests.Length} methods without matching tests!");

        await StoreToJsonFile(outputPath, contextsWithTests);
    }

    private static async Task<Context[]> CreateContexts(string solutionPath, string targetFramework)
    {
        // Roslyn setup
        MSBuildLocator.RegisterDefaults();
        var workspace = MSBuildWorkspace.Create(new Dictionary<string, string> { { "TargetFramework", targetFramework } });
        var solution = await workspace.OpenSolutionAsync(solutionPath);

        // separating projects and files into code and test + basic filtering
        var testProjects = solution.Projects.AsParallel().Where(IsTestProject).ToArray();
        var testFiles = testProjects.AsParallel().SelectMany(p => p.Documents).Where(IsValidDocument).ToArray();

        var mainProjects = solution.Projects.Except(testProjects).ToArray();
        var mainFiles = mainProjects.AsParallel().SelectMany(p => p.Documents).Where(IsValidDocument).ToArray();

        // two step extraction 
        Dictionary<string, Context> contextMap = [];

        // 1. extract methods
        foreach (var file in mainFiles)
        {
            await CodeExtractor.AnalyzeFile(file, contextMap);
        }

        Console.WriteLine($"Failed compilations: {CodeExtractor.errorCounterNull}, \n" +
            $"File contains no class definition: {CodeExtractor.errorCounterClass}, \n" +
            $"Unhandled error during extraction: {CodeExtractor.errorCounterExtraction}");

        // 2. append test information
        foreach (var file in testFiles)
        {
            await TestExtractor.AppendTestInformation(file, contextMap);
        }

        return contextMap.Values.ToArray();
    }

    private static Context[] Filter(Context[] contexts)
    {
        return contexts.Where(c => c.TestsFound).ToArray();
    }

    private static async Task StoreToJsonFile(string outputPath, Context[] contexts)
    {
        const string FILENAME = "extracted.json";
        var options = new JsonSerializerOptions { WriteIndented = true, Encoder = JavaScriptEncoder.Default };

        string path = Path.Combine(outputPath, FILENAME);
        if (File.Exists(path))
            File.Delete(path);

        await using FileStream createStream = File.Create(path);
        await JsonSerializer.SerializeAsync(createStream, contexts, options);

        Console.WriteLine($"{FILENAME} stored at {outputPath}");
    }
}