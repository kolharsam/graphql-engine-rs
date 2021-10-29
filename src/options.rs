use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "graphql-engine-rs",
    about = "An (UNOFFICIAL) implementation of the Hasura GraphQL Engine using rust",
    version = "0.1.0",
    author = "Sameer Kolhar <kolhar.sam@gmail.com>"
)]
pub struct Options {
    #[clap(
        short,
        long,
        about = "given name for the data source",
        default_value = "default"
    )]
    pub source_name: String,
    #[clap(short, long, about = "the connection string to a PG data source")]
    pub connection_string: String,
    #[clap(short, long, about = "server port", default_value = "3000")]
    pub port: u16,
}

pub fn parsed_options() -> Options {
    Options::parse()
}
