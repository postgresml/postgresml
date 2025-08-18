REBAR3 ?= $(shell test -e `which rebar3` 2>/dev/null && which rebar3 || echo "./rebar3")

.PHONY: deps test build

all: build

build: $(REBAR3)
	@$(REBAR3) compile

deps:
	@$(REBAR3) deps

fmt:
	@$(REBAR3) fmt --write

fmt-check:
	@$(REBAR3) fmt --check

shell:
	@$(REBAR3) cargo clean
	@$(REBAR3) shell

clean:
	@$(REBAR3) clean

clean-all:
	rm -rf $(CURDIR)/priv/crates
	rm -rf $(CURDIR)/_build

distclean: clean
	@$(REBAR3) clean --all

docs:
	@$(REBAR3) edoc

test:
	@$(REBAR3) cargo clean
	@$(REBAR3) eunit

test-all:
	@$(REBAR3) do eunit, ct, cover

release: test
	@$(REBAR3) as prod release
