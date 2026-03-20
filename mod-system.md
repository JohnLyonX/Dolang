# Dolang Module System Plan

## Background And Goal

Dolang now distinguishes two module mechanisms with different responsibilities:

- `$mod ...;`
  - imports language modules
  - supports direct paths and wildcard imports
  - exposes module namespaces by file name
- `$HTTP(...).link("...")`
  - mounts HTTP routes from another file
  - only works inside the HTTP subsystem
  - does not import ordinary functions, variables, or constants

This split keeps language reuse and HTTP route mounting decoupled.

## Language Modules: `$mod`

### Syntax

```dol
$mod math.util;
$mod math.*;
```

This phase does not introduce `as`.

### Path Mapping

- module paths use `.` separators
- `math.util` maps to `math/util.dol`
- module namespace defaults to the file name
  - `math.util` becomes namespace `util`
- `std.*` stays reserved for the standard library

### Usage

```dol
$mod math.util;

$ result = util.add(1, 2);
```

Imported functions are not flattened into the global function table.

### Export And Privacy Rules

- `$fn` is public by default
- `_$fn` is private
- `$mod` only exposes public functions
- `$mod` does not import variables, constants, HTTP routes, or `$main`

Example:

```dol
$fn add(a, b) -> Int {
    $# a + b;
}

_$fn helper() -> Int {
    $# 1;
}
```

After import:

- `util.add()` is available
- `util.helper()` is not available

### Wildcard Rules

`$mod math.*;` imports direct child modules only.

Example:

```text
math/
  util.dol
  calc.dol
  internal/
    hidden.dol
```

Import result:

- `util`
- `calc`

Not imported:

- `internal.hidden`

### Conflict Rules

Conflicts fail fast.

The interpreter must reject:

- namespace collisions with existing variables or constants
- namespace collisions with existing functions
- repeated imports that would re-use the same namespace

Errors must identify the conflicting namespace and source file.

## HTTP Route Mounting: `.link()`

### Syntax

```dol
$HTTP("/v1/api").link("routers.api");
```

### Responsibility

`.link()` only:

- resolves the target module file
- scans the file for HTTP routes
- mounts those routes under the current prefix

`.link()` does not:

- import ordinary `$fn`
- expose a module namespace
- replace `$mod`

### Route Extraction

`$HTTP(...).link("routers.api")` collects all HTTP routes in the target file:

- top-level `$GET / $POST / $PUT / $DEL / $PATCH`
- routes inside `$HTTP { ... }` blocks

It ignores:

- `$fn`
- `_$fn`
- `$mod`
- `$main`
- ordinary variables and constants

### Prefix Join Rules

```dol
$HTTP("/v1/api").link("routers.api");
```

If the linked module contains:

```dol
$GET("/users") list_users() -> Json {
    $# "ok";
}
```

The final registered route path is:

```text
/v1/api/users
```

Joining rules:

- trim duplicate `/`
- keep `/` as the root path
- preserve the original route path when the prefix is empty

### `.link()` Limits

- no wildcard support
- missing target module is an error
- linking a file with no HTTP routes is an error

## Execution Plan

### Phase A: Document The Rules

Deliverables:

- this `mod-system.md`
- synchronized updates to `docs/spec/modules.md`
- synchronized updates to HTTP and project-system docs

### Phase B: Implement `$mod`

Required work:

- parser support for `$mod a.b;`
- parser support for `$mod a.*;`
- lexer/parser support for `_$fn`
- runtime module namespace objects keyed by file name
- import only public functions
- reject conflicts immediately

### Phase C: Implement HTTP `.link()`

Required work:

- keep `.link()` inside the HTTP subsystem
- do not reuse ordinary module-import semantics
- collect top-level HTTP routes
- collect routes inside `$HTTP { ... }`
- apply the caller prefix during mounting
- fail when the target module contains no HTTP routes

### Phase D: Clean Up Examples And Specs

Required work:

- update `docs/spec/modules.md`
- update project system docs
- update runtime error docs
- update examples and fixtures away from generic `link("module").fn()`

## Test Plan

### `$mod` Coverage

- single import:
  - `$mod math.util;`
  - `util.add()` is callable
- wildcard import:
  - `$mod math.*;`
  - direct child modules are imported
  - nested directories are not imported
- private functions:
  - `$fn` is visible
  - `_$fn` is hidden from importers
- conflict handling:
  - namespace collisions fail
- export filtering:
  - variables and constants do not enter the module namespace

### HTTP `.link()` Coverage

- top-level HTTP routes are mounted
- routes inside `$HTTP { ... }` are mounted
- prefix joining is correct
- missing target module fails
- module with no HTTP routes fails
- ordinary `$fn` is ignored during mounting

### Regression Coverage

- `$mod` and HTTP `.link()` stay decoupled
- `std.*` resolution keeps working
- old generic `link("module").fn()` behavior is not treated as `$mod`

## Acceptance

This work is complete when:

- `mod-system.md` exists
- `$mod` and `.link()` have clear responsibility boundaries
- `$mod` supports direct paths and wildcard imports
- imported modules are accessed through file-name namespaces
- `_$fn` privacy rules are enforced
- `.link()` only mounts HTTP routes
- the two systems are implemented separately
- conflict handling is explicit
- tests cover both language modules and HTTP route mounting

## Defaults

- module name equals file name
- `$mod` imports functions only
- `_$fn` is the private-function syntax
- `$mod path.*;` scans direct children only
- HTTP `.link()` does not support `*`
- HTTP `.link()` extracts all HTTP routes from the target module
