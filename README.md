# migrate-move

move sql migrations up or down, because that's tedious.

## installing

- install the rust toolchain
- clone this repository
- run `cargo install --path .`

## usage

first run `migrate-move --path <path to migrations> list -o` to list all migrations

Example:

```
$ migrate-move --path migrations/ list -o
  0: 20221025130630_create-schema.up.sql
  1: 20221025131743_create-schema-private.up.sql
  2: 20221025182243_create-table-user.up.sql
  3: 20221025195243_create-table-account.up.sql
```

then run `migrate-move --path up <path to migrations> [up|down] 1` to move the `1`-th migration up|down by one.
This will not print any output.

now run `migrate-move --path <path to migrations> list -o` again, to see the change as it has taken effect

Example:

```
$ migrate-move --path migrations/ list -o
  0: 20221025130630_create-schema-private.up.sql
  1: 20221025131743_create-schema.up.sql
  2: 20221025182243_create-table-user.up.sql
  3: 20221025195243_create-table-account.up.sql
```

notice how in the example the create-schema and create-schema-private have swapped places.

## editors

### neovim
copy `./editors/neovim/migrate-move.lua` to your `$XDG_CONFIG_HOME/nvim/` directory.

before using the plugin, you will have to set it up:

```lua
require('migrate-move').setup({
    program = 'migrate-move',
    up_template = '%d_%s.up.sql', -- %d is a substitute for the index and %s for the name.
    down_template = '%d_%s.down.sql', -- %d is a substitute for the index and %s for the name.
    dir_path = 'migrations', -- the path of the migrations directory, relative to your working dir
})
```
