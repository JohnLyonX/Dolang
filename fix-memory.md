# Dolang Memory Fix Plan

## Scope

- Goal: investigate and plan fixes for the `dolang serve` memory spike that can reach multi-GB RSS.
- Excluded from this review: all in-flight `runtime/auth` work and related auth-only modules.
- Files intentionally excluded from change planning:
  - `crates/dolang-runtime/src/runtime/auth/**`
  - auth-focused code paths in stdlib modules
- This document is a repair plan, not an implementation.

## Review Summary

The current memory risk is structural, not a single leak site. The main problem is that route registration and request execution repeatedly deep-clone large interpreter/runtime state.

The highest-risk copy chain is:

1. `HttpRoute` stores full handler body plus full module env/function snapshots.
2. `RuntimeContext` stores `Vec<HttpRoute>` directly.
3. `clone_for_request_execution()` deep-clones the full route/static/type registries.
4. Axum backend clones one full `RuntimeContext` per route at startup, then another full clone per request.
5. Request execution clones route-level `module_env` and `module_fns` again into a fresh `ProgramState`.

This creates multiplicative growth:

- per module: route count multiplies module snapshot size
- per server startup: route count multiplies runtime context size
- per request: full route graph and module snapshots get copied again

## Confirmed Findings

### 1. `RuntimeContext` deep-clones route and type registries per request

Evidence:

- [crates/dolang-runtime/src/runtime/context.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/context.rs#L141)
- [crates/dolang-runtime/src/runtime/context.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/context.rs#L149)

`clone_for_request_execution()` delegates to `clone_with_sql_conn_registry()`, which currently clones:

- `http_routes`
- `static_routes`
- `native_fn_registry`
- `native_module_registry`
- `type_registry`

This means request-local execution is carrying a fresh copy of global routing/type data that should be immutable and shared after boot.

### 2. Axum clones a full runtime context once per route and once per request

Evidence:

- [crates/dolang-cli/src/backends/axum_backend.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-cli/src/backends/axum_backend.rs#L201)
- [crates/dolang-cli/src/backends/axum_backend.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-cli/src/backends/axum_backend.rs#L207)

Inside `build_router()`:

- startup path: `Arc::new(context.clone_for_request_execution())` for every route
- request path: `handler_context.clone_for_request_execution()` for every request

This is the largest single multiplier in the serving path.

### 3. Every `HttpRoute` embeds a full module snapshot

Evidence:

- [crates/dolang-runtime/src/interpreter/mod.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/interpreter/mod.rs#L12)
- [crates/dolang-runtime/src/interpreter/exec/http.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/interpreter/exec/http.rs#L20)
- [crates/dolang-runtime/src/interpreter/exec/http.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/interpreter/exec/http.rs#L213)

`HttpRoute` currently includes:

- `body: Vec<Stmt>`
- `module_env: Env`
- `module_fns: FnEnv`

Routes created by `handle_http_fn()`, `handle_http_block()`, and `load_module_routes()` all clone the module state into each route. If one module registers many routes, each route carries the same environment/function graph again.

### 4. Request execution clones route snapshots yet again

Evidence:

- [crates/dolang-runtime/src/runtime/http.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/runtime/http.rs#L49)

`execute_http_route_in_context()` clones:

- `route.module_env`
- `route.module_fns`

into a fresh `ProgramState` for every request. That is correct semantically, but the source data being cloned is too large because the route owns full module snapshots.

### 5. Function metadata is also heavy

Evidence:

- [crates/dolang-runtime/src/interpreter/env.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-runtime/src/interpreter/env.rs#L55)

`RuntimeFn` includes:

- full `FnDeclStmt`
- `source_file`
- `module_env`

So cloning `FnEnv` is not cheap. Repeating that per route amplifies memory quickly.

### 6. Startup path does extra full-vector copies

Evidence:

- [crates/dolang-cli/src/server.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-cli/src/server.rs#L46)
- [crates/dolang-cli/src/server.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-cli/src/server.rs#L68)
- [crates/dolang-cli/src/server.rs](/Users/liangzhanbo/CodeStudio/dolang/crates/dolang-cli/src/server.rs#L81)

`context.routes().to_vec()` and `context.static_routes().to_vec()` add more startup copying. This is not the main 5GB driver, but it raises peak memory and should be removed after the main fix lands.

## Non-Primary Suspects

These areas may contribute to memory, but they are not the first repair target:

- request body buffering in `axum_backend.rs`
- auth/session state
- SQL connection registry
- static file serving

The structural route/context cloning problem is much more likely to explain a persistent 5GB footprint.

## Repair Plan

### Phase 1. Stop deep-cloning global runtime state

Target files:

- `crates/dolang-runtime/src/runtime/context.rs`
- `crates/dolang-cli/src/backends/axum_backend.rs`
- `crates/dolang-cli/src/server.rs`

Changes:

- Convert immutable runtime registries to shared storage after boot:
  - `http_routes`
  - `static_routes`
  - `type_registry`
  - possibly native registries if needed for clone cost
- Replace request-time deep clones with cheap shared clones using `Arc`.
- Split runtime context into:
  - shared immutable serve-time data
  - request-local mutable state
- Remove per-route `context.clone_for_request_execution()` in `build_router()` and replace it with a shared `Arc<RuntimeContextShared>` or equivalent wrapper.
- Remove redundant `to_vec()` startup copies in `server.rs` once route ownership is reworked.

Success criteria:

- cloning a request context does not duplicate the route graph
- router build memory no longer grows linearly with `route_count * full_context_size`

### Phase 2. Make route module state shared instead of embedded per route

Target files:

- `crates/dolang-runtime/src/interpreter/mod.rs`
- `crates/dolang-runtime/src/interpreter/exec/http.rs`
- `crates/dolang-runtime/src/runtime/http.rs`
- `crates/dolang-runtime/src/interpreter/env.rs`

Changes:

- Remove direct ownership of full `module_env` / `module_fns` from `HttpRoute`.
- Introduce a shared route execution snapshot, for example:
  - `Arc<RouteModuleState>`
  - or a module snapshot registry keyed by module id
- Ensure multiple routes from the same module point to the same immutable module snapshot.
- Keep request isolation by cloning only the minimal per-request env/fn layer needed by execution.

Success criteria:

- N routes from one module no longer store N copies of the same module env/fn graph
- per-request state seeding remains semantically isolated

### Phase 3. Reduce function-environment duplication

Target files:

- `crates/dolang-runtime/src/interpreter/env.rs`
- function declaration/load paths under `crates/dolang-runtime/src/interpreter/exec/**`

Changes:

- Review whether `RuntimeFn.module_env` needs full ownership or can point to shared module-level state.
- Consider storing function declarations and closure/module env separately:
  - immutable declaration in shared storage
  - small closure snapshot only when necessary
- Avoid recursive duplication where route snapshot contains function env, and each runtime function also contains another full module env.

Success criteria:

- `FnEnv` clone size becomes proportional to function references, not repeated full module state

### Phase 4. Add regression coverage focused on memory shape

Target files:

- runtime unit tests
- server/backend tests

Changes:

- Add a synthetic many-route test that exercises module linking and route registration.
- Add clone-cost regression tests that assert shared storage semantics rather than RSS:
  - route registry pointer identity
  - snapshot sharing across routes from the same module
  - request clone does not increase registry instance count
- If practical, add an ignored benchmark or debug command to compare:
  - route registration count
  - snapshot counts
  - allocation-sensitive workload before/after

Success criteria:

- future refactors cannot silently reintroduce route/context deep cloning

## Recommended Execution Order

1. Phase 1 first
2. Phase 2 second
3. Phase 3 only after Phase 2 data shape is stable
4. Phase 4 after each structural phase to lock behavior in

This order matters because Phase 1 removes the biggest serve-time multiplier without colliding with `runtime/auth`, and Phase 2 then removes the biggest route-shape multiplier.

## Risks

- `RuntimeContext` currently mixes mutable request state and immutable global registries; splitting it will touch multiple call sites.
- Route registration currently assumes owned `Vec<HttpRoute>` storage; moving to shared storage may require a boot/freeze boundary.
- Module snapshot sharing must preserve Dolang execution semantics for `$mod`, `$fn`, and linked router modules.
- Type registry merging in linked modules currently copies the whole registry; that path will need careful redesign to avoid restoring a deep-copy problem in a new place.

## Out of Scope For This Fix

- auth/session architecture changes
- HTTP feature additions
- SQL/runtime data-path optimizations unrelated to route/context cloning
- frontend or sample-project issues

## Expected Outcome

After these phases:

- server startup should no longer allocate one full runtime-context copy per route
- steady-state request handling should no longer duplicate the full route registry
- linked router modules should share one module snapshot instead of embedding it into every route
- memory growth should become closer to:
  - global shared state
  - plus small per-request execution state
  - instead of global state multiplied by route count and request count
