use clap::Clap;

#[derive(Clap, Debug, Clone)]
#[clap(
    name = "graphql-engine-rs",
    about = "An implementation of the Hasura GraphQL Engine using rust",
    version = "0.1.0",
    author = "Sameer Kolhar <sameer@hasura.io>"
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
    // TODO: consider to put this in later, maybe not just yet
    // #[clap(
    //     long,
    //     default_value = "[\"public\"]",
    //     about = "the schema(s) that should be scanned to generate the GraphQL schema"
    // )]
    // pub schemas: Vec<String>,
    #[clap(short, long, about = "server port", default_value = "3000")]
    pub port: u16,
}

pub fn parsed_options() -> Options {
    Options::parse()
}
