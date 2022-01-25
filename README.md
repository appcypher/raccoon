<div align="center">
    <a href="#" target="_blank">
        <img src="https://raw.githubusercontent.com/appcypher/gigamono-assets/main/avatar-gigamono-boxed.png" alt="Gigamono Logo" width="140" height="140"></img>
    </a>
</div>

<h1 align="center">Compiler Workflow</h1>

`compiler_workflow` takes a **workflow graph** (`.wg`) as input, runs semantic analysis on it and generates JavaScript or WebAssembly code and source maps.

The workflow graph uses a typed language and its semantics is based on [Raccoon](https://github.com/appcypher/raccoon).

When generating wasm code, any JavaScript module dependency is accessed via the host interface `gigamono/js_call` which is expected to be implemented by a Gigamono implementation.
