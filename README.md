[![Board Status](https://dev.azure.com/kolharsam/fe7c384b-88c9-432a-b06c-e930b83bd6a0/39a97781-2ff6-4144-87ec-c14e36094309/_apis/work/boardbadge/6ee40c04-0147-4e72-bc83-73f47e8e9e55)](https://dev.azure.com/kolharsam/fe7c384b-88c9-432a-b06c-e930b83bd6a0/_boards/board/t/39a97781-2ff6-4144-87ec-c14e36094309/Microsoft.RequirementCategory)
# graphql-engine-rs

[![Build & Test Workflow badge](https://github.com/kolharsam/graphql-engine-rs/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/kolharsam/graphql-engine-rs/actions/workflows/rust.yml)

The Hasura GraphQL Engine - in Rust

This is not official. This is just a toy project. 

We can run GraphQL queries as of now. More features related to the metadata, GraphQL schema (with introspection support) subscriptions and so on...will be arriving soon!

Here's a short clip of the same:

https://user-images.githubusercontent.com/6604943/129950893-263d5785-3552-4f87-a15b-f8b4add670ac.mov

##### FOR RUNNING THE TEST SUITE

- Use a env. var named `DATABASE_URL` to set the database that'll be used to run the tests.
- Make sure to run the set up the database schema(s) for the tests from [here](/migrate/schema.sql).

More Documentation and Implementation details will be published soon!
