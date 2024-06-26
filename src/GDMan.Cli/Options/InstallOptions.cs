using GDMan.Cli.Attributes;
using GDMan.Core.Helpers;
using GDMan.Core.Models;

namespace GDMan.Cli.Options;

[Command("install", "i", "Installs the specified version of Godot")]
public class InstallOptions : BaseOptions
{
    private static readonly SemanticVersioning.Range MinLinuxVersion = new(">=4.0.0");


    [Option("latest", "l", "Whether or not the latest version should be fetched. "
        + "If used in conjunction with the Version argument and multiple matching "
        + "versions are found, the latest of the matches will be used. ", OptionDataType.Boolean, isFlag: true)]
    public bool Latest { get; set; } = false;

    [Option("version", "v", "The version to use, e.g. 1.2.3. Any valid semver range is supported", OptionDataType.String)]
    public SemanticVersioning.Range? Version { get; set; } = null;

    [Option("platform", "p", "The platform or operating system to find a version for", OptionDataType.Enum)]
    public Platform Platform { get; set; } = PlatformHelper.FromEnvVar() ?? PlatformHelper.FromSystem();

    [Option("architecture", "a", "The system architecture to find a version for", OptionDataType.Enum)]
    public Architecture Architecture { get; set; } = ArchitectureHelper.FromEnvVar() ?? ArchitectureHelper.FromSystem();

    [Option("flavour", "fl", "The \"flavour\" (for lack of a better name) of version to use", OptionDataType.Enum)]
    public Flavour Flavour { get; set; } = FlavourHelper.FromEnvVar() ?? Flavour.Standard;

    public override OptionValidation Validate()
    {
        if (Version == null && !Latest)
        {
            return OptionValidation.Failed("Either --version or --latest must be provided");
        }

        if (Platform == Platform.Windows && (Architecture == Architecture.Arm32 || Architecture == Architecture.Arm64))
        {
            return OptionValidation.Failed($"Architecture {Architecture} not supported on Windows platform");
        }

        if (Platform == Platform.Linux && Version != null && Version.Intersect(MinLinuxVersion) != Version)
        {
            return OptionValidation.Failed("GDMan does not support Godot version < 4 on Linux");
        }

        return OptionValidation.Success();
    }
}