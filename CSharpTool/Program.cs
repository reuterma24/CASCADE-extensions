class Program
{
    const string EXTRACTION = "extract";
    const string MODIFICATION = "modify";
    const string VERIFICATION = "verify";

    static async Task<int> Main(string[] args)
    {
        if (args.Length < 1)
            return InvalidInvocationMessage();

        switch (args[0])
        {
            case EXTRACTION:
                await Extractor.Extract(args[1..]);
                break;

            case MODIFICATION:
                Modifier.Modify(args[1..]);
                break;

            case VERIFICATION:
                var result = Verifier.Verify(args[1..]);
                return result;

            default:
                return InvalidInvocationMessage();

        }

        return 0;
    }

    private static int InvalidInvocationMessage()
    {
        Console.WriteLine("Usage: cSharpTool <command>, (possible commands: extract, modify, verify)");
        return -1;
    }
}