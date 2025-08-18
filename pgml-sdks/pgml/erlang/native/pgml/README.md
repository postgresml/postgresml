# NIF for pgml

## To compile the NIF in Erlang:

Add the following lines to your `rebar.config` file:

``` erlang
{plugins, [rebar3_rustler]}.

{cargo_opts, [
    {src_dir, "native/pgml"}
]}.

{provider_hooks, [
    {pre, [
        {compile, {cargo, build}}
    ]},
    {post, [
        {clean, {cargo, clean}},
        {eunit, {cargo, test}}
    ]}
]}.
```

## Build

``` shell
cargo build
```

## Test

``` shell
cargo test
```
