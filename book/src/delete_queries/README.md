# Delete Queries
Writing delete queries, which is pretty much the same as [Select Queries](../select_queries/index.html), is incredibly easy with the `delete_query` macro.

Simply create a struct with the fields you want to select, and annotate it with the `#[write_query]` macro.

```rust
#use scyllax::prelude::*;
#
#\#[entity]
#pub struct PersonEntity {
#	#[entity(pk)]
#    pub id: uuid::Uuid,
#    pub email: String,
#	#[entity(rename = "createdAt")]
#    pub created_at: i64,
#}
#
#[write_query(
    query = "delete from person where id = ?",
    entity_type = "PersonEntity"
)]
pub struct DeletePersonById {
    pub id: Uuid,
}
```

Then, you can pass it to the executor you made in [Introduction](../index.html).
```rust,ignore
let query = DeletePersonById {
    id: Uuid::from_str("00000000-0000-0000-0000-000000000000")?,
};

let res = executor
    .execute_delete(query)
    .await?
```
