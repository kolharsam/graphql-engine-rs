CREATE TABLE authors(
  id SERIAL PRIMARY KEY NOT NULL,
  author_name TEXT NOT NULL
);

INSERT INTO authors (author_name) VALUES ('sam');
INSERT INTO authors (author_name) VALUES ('bam');
INSERT INTO authors (author_name) VALUES ('can');
INSERT INTO authors (author_name) VALUES ('of');
INSERT INTO authors (author_name) VALUES ('ham');
