<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [GrandLine](#grandline)
  - [Docs](#docs)
  - [Contributing](#contributing)
  - [License: MIT](#license-mit)

<!-- END doctoc -->

# GrandLine

Rust macro framework for building GraphQL APIs on top of `sea-orm` and `async-graphql` - fine granted dynamic authorization per col/row levels, automatic CRUD resolvers, nested filtering, sorting, pagination, relationships, and soft-delete.

<p align="center">
  <img src="https://github.com/namnm/grand-line/blob/master/.md/banner.jpg?raw=true" alt="Grand Line One Piece"/>
</p>

- [Simple Todo example](https://github.com/namnm/grand-line/blob/master/examples/simple_todo/src/lib.rs)
- [Saas example (auth + authz)](https://github.com/namnm/grand-line/blob/master/examples/saas)
- [All examples](https://github.com/namnm/grand-line/blob/master/examples)
- [Tests](https://github.com/namnm/grand-line/blob/master/tests)

## Docs

- [Quick start](docs/quick-start.md)
- [Model: auto-generated types, auto-added fields, field attributes](docs/model.md)
- [CRUD resolvers: search, count, detail, create, update, delete](docs/crud-resolvers.md)
- [Custom resolvers: plain `#[query]`/`#[mutation]`](docs/custom-resolvers.md)
- [Relationships: has_one, has_many, belongs_to, many_to_many, custom relation resolvers](docs/relationships.md)
- [Filtering and sorting](docs/filtering-sorting.md)
- [Schema collector: `grand_line_build`, no more manual `MergedObject`](docs/schema-collector.md)
- [Subscriptions: `#[subscribe]`, in memory or redis broker](docs/subscriptions.md)
- [Resolvers, context, and transactions](docs/resolvers.md)
- [Active model helpers: `am_create!`, `am_update!`, `am_soft_delete!`](docs/active-model-helpers.md)
- [History: opt-in per-model audit log](docs/history.md)
- [SeaORM query helpers](docs/query-helpers.md)
- [Error handling: `#[grand_line_err]`, `#[client]`](docs/error-handling.md)
- [Authentication: session + OTP primitives, build your own register/login/forgot](docs/authentication.md)
- [Authorization: org scoping, fine granted dynamic col/row levels](docs/authorization.md)
- [Debug macro outputs: see the code a macro actually generated](docs/debug-macros.md)

## Contributing

Design notes (strengths, known limitations, roadmap) and dev setup live in [docs/contribution.md](docs/contribution.md).

## License: MIT

Contact: [nam@namnm.com](mailto:nam@namnm.com)
