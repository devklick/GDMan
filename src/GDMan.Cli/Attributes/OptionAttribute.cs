using System.Reflection;

using GDMan.Cli.Options;
using GDMan.Core.Attributes;
using GDMan.Core.Extensions;

namespace GDMan.Cli.Attributes;

[AttributeUsage(AttributeTargets.Property)]
public class OptionAttribute : Attribute
{
    public string FullName { get; }
    public string ShortName { get; }
    public string Description { get; }
    public OptionDataType Type { get; }
    public bool IsFlag { get; }

    public OptionAttribute(string fullName, string shortName, string description, OptionDataType type, bool isFlag = false)
    {
        FullName = fullName;
        ShortName = shortName;
        Description = description;
        Type = type;
        IsFlag = isFlag;

        if (IsFlag && Type != OptionDataType.Boolean)
            throw new InvalidOperationException("CLI args which are flags must be of boolean type");

        if (!FullName.StartsWith("--")) FullName = $"--{FullName}";
        if (!ShortName.StartsWith('-')) ShortName = $"-{ShortName}";
    }

    public virtual OptionValidation Validate(PropertyInfo propertyInfo, object? value) => Type switch
    {
        OptionDataType.String => ValidateString(propertyInfo, value),
        OptionDataType.Boolean => ValidateBoolean(propertyInfo, value),
        OptionDataType.Enum => ValidateEnum(propertyInfo, value),
        _ => throw new NotImplementedException($"Unsupported type ${Type} for CliArgAttribute")
    };

    private static OptionValidation ValidateString(PropertyInfo propertyInfo, object? value)
    {
        if (propertyInfo.PropertyType == typeof(SemanticVersioning.Range))
        {
            return SemanticVersioning.Range.TryParse(value?.ToString(), out var semVer)
                ? OptionValidation.Success(semVer)
                : OptionValidation.Failed();
        }
        return OptionValidation.Success(value?.ToString() ?? "");
    }

    private OptionValidation ValidateBoolean(PropertyInfo _, object? value)
    {
        if (IsFlag) return OptionValidation.Success(true);

        if (!bool.TryParse((string)value!, out var boolValue))
        {
            return OptionValidation.Failed();
        }
        return OptionValidation.Success(boolValue);
    }

    private static OptionValidation ValidateEnum(PropertyInfo propertyInfo, object? value)
    {
        if (value == null) return OptionValidation.Failed();

        var values = Enum.GetValues(propertyInfo.PropertyType);

        foreach (var e in values)
        {
            // If the specified value matches the enum name, regardless of case
            if (e.ToString()?.ToLower() == value.ToString()?.ToLower())
                return OptionValidation.Success(e);

            // If the specified value matches one of the enum aliases, regardless of case
            var aliasAttr = ((Enum)e).GetAttribute<AliasAttribute>();
            foreach (var alias in aliasAttr?.Aliases ?? [])
            {
                if (value.ToString()?.ToLower() == alias.ToLower())
                {
                    return OptionValidation.Success(e);
                }
            }
        }

        return OptionValidation.Failed();
    }
}