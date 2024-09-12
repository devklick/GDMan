<h1 align="center">
    gdman
</h1>

<p align="center">
    An unofficial version manager for Godot
</p>
<br/>

Godot Version Manager (aka gdman) is an cross-platform command-line application for managing your installed version(s) of the [Godot game engine](https://github.com/godotengine/godot).

Supported commands:
- [`install`](#install-command)
- [`update`](#uninstall-command)
- [`uninstall`](#update-command)
- [`current`](#current-command)
- [`list`](#list-command)

### Install gdman

To install gdman, head over to the [latest release](https://github.com/devklick/gdman-rs/releases/latest) and use one of the install scripts.

Things to note:

- gdman will be installed to `~/.gdman/`, and this folder will be added to your PATH
- After running `gdman install`, you'll have a `godot` symlink that you can invoke from the terminal to launch the currently-active version of godot (note that this will be godot.lnk on Windows).
- All versions of Godot installed via gdman will be in `~/.gdman/versions`

It can also be installed via [Cargo](https://crates.io/crates/gdman).

### Updating gdman

To update GDMan you can just repeat the installation process above for a different version.

### Install Command

To install a version of godot, use the `gdman install` command. You need to specify at least one of two arguments:
- `--version` (`-v`) - to install an exact version or a version matching an input semver constraint
- `--latest` (`-l`) - to install the latest version

#### Install mono versions

If you need to install a mono version of Godot, you can pass in the `--flavour` (`-f`) argument with a value of `mono`.

**Note**, you can use the install command to switch to another version of Godot that's currently installed on the system. Before attempting to install a version, gdman will check if the version you want is already installed and, if so, will activate it.

#### Using the `--version` argument

There are two ways to use this argument:

##### Looking for exact versions

If you just pass in a semver version, e.g. `4`, `4.2`, `4.2.1`, gdman will look for that version and, if found, will install it.

##### Looking for similar versions

- You can specify a semver range, e.g. `">=4.1, <4.3"`. This will find and install the latest versions which is greater or equal to 4.1 and less than 4.3.
- You can specify a semver patch constraint, e.g `~4.1`. This will install the latest patch version of 4.1 (at the time of writing this, it's 4.1.4).
- You can specify a minor patch constraint, e.g. `^4`. This will find the latest minor patch for version 4 (at the time of writing this, it's 4.3).


For more info, run `gdman install --help`.

### Uninstall Command

If you need to uninstall versions of Godot that you have previously installed using gdman, you can use the `gdman uninstall` command. It's similar to the `install` command in that:
- You can specify a semver constraint via the `--version` (`-v`)argument to uninstall one or more versions matching a set constraint. 
- You can uninstall a specific flavour (mono, standard) via the the `--flavour` (`-f`) argument

Alternatively, if just want to remove *all* versions of Godot apart from the one which is currently active on the system, you can use the `--unused` argument.

For more info, run `gdman uninstall --help`.

### Update Command

To conveniently update the version of Godot that you currently have set active on your system, you can use the `gdman update` command. This allows you to perform one of three types of update:
- `--patch` - To update to the latest patch of the current version. E.g. if you're on v1.2.3, v1.2.4 and v1.3 are available, this will update you to v1.2.4.
- `--minor` - To update to the latest minor revision of the current version. E.g. if you're on v1.2.3, v1.3.4 and v2 are available, this will update you to v1.3.4.
- `--major` - To update to the latest version of the current version. E.g. if you're on v1.2.3, v1.3.4 and v2.1.0 are available, this will update you to v2.1.0.

**Note** that the `gdman update` command will install and set active the relevant version of Godot. It does not remove the version that's installed at the time of running the command.

For more info, run `gdman update --help`.

### Current Command

One of the benefits of using gdman is that a single godot shortcut is used - there's one version of Godot active on the system at any one time. 

To list the version of Godot that is currently active on the system, you can use the `gdman current` command.

For more info, run `gdman current --help`.

### List Command

To list the versions of Godot that are currently installed on the system, you can use the `gdman list` command. 

For more info, run `gdman list --help`.
