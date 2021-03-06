CREATE TABLE authors(
  id SERIAL PRIMARY KEY NOT NULL,
  author_name TEXT NOT NULL
);

INSERT INTO authors (author_name) VALUES ('sam');
INSERT INTO authors (author_name) VALUES ('bam');
INSERT INTO authors (author_name) VALUES ('can');
INSERT INTO authors (author_name) VALUES ('of');
INSERT INTO authors (author_name) VALUES ('ham');

CREATE SCHEMA "new_schema";

CREATE TABLE "new_schema"."users"(
  "user_id" SERIAL PRIMARY KEY NOT NULL,
  "name" TEXT NOT NULL,
  age INT,
  comment TEXT
);

INSERT INTO "new_schema"."users" ("name") VALUES ('sam');
INSERT INTO "new_schema"."users" ("name", age) VALUES ('bam', 24);
INSERT INTO "new_schema"."users" ("name", comment) VALUES ('can', 'hello world');
INSERT INTO "new_schema"."users" ("name", age, comment) VALUES ('of', 23, 'this is a comment');
INSERT INTO "new_schema"."users" ("name") VALUES ('ham');
