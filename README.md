# graphql-engine-rs

The Hasura GraphQL Engine - in Rust

This is not official. This is just a toy project. 

We can run GraphQL queries as of now. More features related to the metadata, GraphQL schema (with introspection support)

Here's a short clip of the same:

https://user-images.githubusercontent.com/6604943/129950893-263d5785-3552-4f87-a15b-f8b4add670ac.mov

##### FOR RUNNING THE TEST SUITE

- Use a env. var named `DATABASE_URL` to set the database that'll be used to run the tests.
- Make sure to run the set up the database schema(s) for the tests from #[here](/test/schema.sql).

More Documentation and Implementation details will be published soon!
