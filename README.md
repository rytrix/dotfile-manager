# dotfile-manager

A simple rust application to manage dotfiles through symlinks

### Usage
```
Usage: dm [OPTIONS]

Options:
  -f, --file <file>              Config file to load, default dots.toml
  -d, --deploy <config>         Deploy a config
  -c, --clean
      --clean-config <config>    Clean a specified config
  -l, --list                   Display all entries
      --list-full              Display full config
  -r, --dry-run                Dry run
  -h, --help                   Print help
  -V, --version                Print version
```


### Example dots.toml

```toml
title = "user's dots"
src_dir = "~/Documents/dotfiles"
dst_dir = "~/.config"

[debian]
# dst_dir/nvim -> src_dir/nvim
nvim = "nvim"
sway = "sway-debian"
# full-path -> src_dir/pure-starship.toml
"~/.local/share/fonts" = "fonts"
"~/.local/bin/lockscreen" = "lockscreen.sh"

[arch]
# dst_dir/nvim -> src_dir/nvim
nvim = "nvim"
sway = "sway-arch"
# full-path -> src_dir/fonts
"~/.local/share/fonts" = "fonts"
"~/.local/bin/lockscreen" = "lockscreen.sh"
```