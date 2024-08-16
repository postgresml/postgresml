-cargo_header_version(1).
-ifndef(CARGO_LOAD_APP).
-define(CARGO_LOAD_APP, pgml).
-endif.
-ifndef(CARGO_HRL).
-define(CARGO_HRL, 1).
-define(load_nif_from_crate(__CRATE, __INIT),
    (fun() ->
        __APP = ?CARGO_LOAD_APP,
        __PATH = filename:join([code:priv_dir(__APP), "crates", __CRATE, __CRATE]),
        erlang:load_nif(__PATH, __INIT)
    end)()
).
-endif.
