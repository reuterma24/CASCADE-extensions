using System.Text.Json.Serialization;

public class Context
{
    [JsonIgnore]
    public string RootPath { get; set; } = string.Empty;

    [JsonPropertyName("doc")]
    public required string Doc { get; set; }

    [JsonPropertyName("signature")]
    public required Signature Signature { get; set; }

    [JsonPropertyName("language")]
    public required string Language { get; set; }

    [JsonPropertyName("parent")]
    public required Parent Parent { get; set; }

    [JsonPropertyName("code")]
    public required string Code { get; set; }

    [JsonPropertyName("code_file_path")]
    public required string CodeFilePath { get; set; }

    [JsonPropertyName("called_functions")]
    public required string[] CalledFunctions { get; set; }

    [JsonPropertyName("id")]
    public uint Id { get; set; }

    [JsonPropertyName("tests")]
    public List<Test> Tests { get; set; } = [];

    [JsonIgnore]
    public bool TestsFound => Tests.Count > 0;

    [JsonPropertyName("new_tests")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)]
    public string? NewTests { get; set; } = default;

    [JsonPropertyName("new_code")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingDefault)]
    public string? NewCode { get; set; } = default;

    public Context DeepCopy()
    {
        var copy = (Context)this.MemberwiseClone();
        copy.Signature = this.Signature.DeepCopy();
        copy.Parent = this.Parent.DeepCopy();

        return copy;
    }
}

public class Signature
{
    [JsonPropertyName("name")]
    public required string Name { get; set; }

    [JsonPropertyName("returns")]
    public required string Returns { get; set; }

    [JsonPropertyName("params")]
    public required string[] Params { get; set; }

    [JsonPropertyName("modifier")]
    public required string[] Modifier { get; set; }

    [JsonPropertyName("annotations")]
    public required string[] Annotations { get; set; }

    [JsonPropertyName("generics")]
    public required string[] Generics { get; set; }

    public Signature DeepCopy()
    {
        return (Signature)this.MemberwiseClone();
    }

    public override bool Equals(object? obj)
    {
        if (obj is not Signature other)
            return false;

        return Name == other.Name
            && Returns == other.Returns
            && Params.SequenceEqual(other.Params)
            && Modifier.SequenceEqual(other.Modifier)
            && Annotations.SequenceEqual(other.Annotations)
            && Generics.SequenceEqual(other.Generics);
    }

    public override int GetHashCode()
    {
        return HashCode.Combine(
        Name,
        Returns,
        string.Join(",", Params),
        string.Join(",", Modifier),
        string.Join(",", Annotations),
        string.Join(",", Generics)
        );
    }

    public static bool operator ==(Signature? left, Signature? right)
    {
        // Check if both are null or if one is null
        if (left is null)
            return right is null;

        return left.Equals(right);
    }

    public static bool operator !=(Signature? left, Signature? right)
    {
        return !(left == right);
    }
}

public class Parent
{
    [JsonPropertyName("name")]
    public required string Name { get; set; }

    [JsonPropertyName("doc")]
    public required string Doc { get; set; }

    [JsonPropertyName("other_methods")]
    public required OtherMethod[] OtherMethods { get; set; }

    [JsonPropertyName("variables")]
    public required string[] Variables { get; set; }

    [JsonPropertyName("generics")]
    public required string[] Generics { get; set; }

    [JsonPropertyName("imports")]
    public required string[] Imports { get; set; }

    [JsonPropertyName("constructors")]
    public required string[] Constructors { get; set; }

    [JsonPropertyName("extends")]
    public required string Extends { get; set; }

    [JsonPropertyName("implements")]
    public required string[] Implements { get; set; }

    [JsonPropertyName("namespace")]
    public required string Namespace { get; set; }

    public Parent DeepCopy()
    {
        var copy = (Parent)this.MemberwiseClone();
        copy.OtherMethods = [.. this.OtherMethods];

        return copy;
    }
}

public class OtherMethod
{
    [JsonPropertyName("doc")]
    public required string Doc { get; set; }

    [JsonPropertyName("signature")]
    public required Signature Signature { get; set; }

    [JsonPropertyName("code")]
    public required string Code { get; set; }
}

public class Test
{
    [JsonPropertyName("tests")]
    public string Tests { get; set; } = string.Empty;

    [JsonPropertyName("test_imports")]
    public string[] TestImports { get; set; } = [];

    [JsonPropertyName("test_namespace")]
    public string TestNamespace { get; set; } = string.Empty;

    [JsonPropertyName("test_class_name")]
    public string TestClassName { get; set; } = string.Empty;

    [JsonPropertyName("test_file_path")]
    public string TestFilePath { get; set; } = string.Empty;

    [JsonPropertyName("test_runner")]
    public string TestRunner { get; set; } = string.Empty;

    [JsonPropertyName("project_path")]
    public string ProjectPath { get; set; } = string.Empty;
}