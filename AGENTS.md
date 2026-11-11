## Conventions

## Quality Checklist (Rails-style)

When writing your code:

1. **Is it in the right place?** Business logic in the Model, flow control in the Controller.
2. **Is it DRY?** Avoid repeating the same logic across multiple files.
3. **Is it Secure?** Use *Strong Parameters* (validated DTOs) to prevent attacks.
4. **Is it Tested?** Ensure there is a unit test validating this logic.
5. Readability (code as prose).
6. Principle of Least Surprise (POLS).
7. Self-sufficiency (the "No-PaaS" philosophy): the system runs self-hosted, with no mandatory external services required to function.
8. High cohesion and loose coupling. Do not use single-letter variable names (e.g., a, b, c, d).
9. Code, file names, and comments must be in English.
10. Commits must follow standard conventional commits in English.
11. Hardcoded text must use i18n keys.

### Things to Avoid

- Em dashes and decorative dividers in text, comments, and code: `—`, `————`, `---`, ASCII art (detail in 1. Philosophy, line "No decorative dividers").
- `unsafe` in Rust and `unwrap()` outside of tests.
- `CREATE TYPE ENUM` and `CHECK` constraints with a value list on new columns (see 4. Database Conventions).

### Things to Prioritize

- Normalized and optimized tables, with proper field types (>=3NF, see 4. Database Conventions).
- Non-boolean database columns that have more than two options must be stored as a SMALLINT enum, Rails-style:

```ruby
enum :status, { pending: 0, in_progress: 1, completed: 2 }
```

Rust equivalent:

```rust
pub enum Status {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
}
```

The Rust model is the single source of truth: the database does not validate the list of values, and a legacy CHECK constraint is removed via migration when the enum receives a new value (never expanded).
