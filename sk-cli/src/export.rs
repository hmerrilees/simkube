use anyhow::bail;
use sk_api::v1::{
    ExportFilters,
    ExportRequest,
    ExportFormat,
};
use sk_core::external_storage::{
    ObjectStoreWrapper,
    SkObjectStore,
};
use sk_core::prelude::*;
use sk_core::time::duration_to_ts;

#[derive(clap::ValueEnum, Clone)]
pub enum ExportFormatArg {
    Json,
    Msgpack,
}

impl From<ExportFormatArg> for ExportFormat {
    fn from(value: ExportFormatArg) -> Self {
        match value {
            ExportFormatArg::Json => ExportFormat::Json,
            ExportFormatArg::Msgpack => ExportFormat::Msgpack,
        }
    }
}

#[derive(clap::Args)]
pub struct Args {
    #[arg(
        short,
        long,
        long_help = "trace export start timestamp; can be a relative duration\n\
             or absolute timestamp; durations are computed relative\n\
             to the specified end time, _not_ the current time",
        default_value = "-30m",
        value_parser = duration_to_ts,
        allow_hyphen_values = true,
    )]
    pub start_time: i64,

    #[arg(
        short = 't',
        long,
        long_help = "end time; can be a relative or absolute timestamp",
        default_value = "now",
        value_parser = duration_to_ts,
        allow_hyphen_values = true,
    )]
    pub end_time: i64,

    #[arg(
        long,
        long_help = "namespaces to exclude from the trace",
        value_delimiter = ',',
        default_value = "cert-manager,kube-system,local-path-storage,monitoring,simkube"
    )]
    pub excluded_namespaces: Vec<String>,

    #[arg(
        long,
        long_help = "sk-tracer server address",
        default_value = "http://localhost:7777"
    )]
    pub tracer_address: String,

    #[arg(
        short,
        long,
        long_help = "location to save exported trace",
        default_value = "file:///tmp/kind-node-data"
    )]
    pub output_path: String,

    #[arg(short, long, long_help = "format in which to export trace data", default_value = "json")]
    pub format: ExportFormatArg,
}

pub async fn cmd(args: &Args) -> EmptyResult {
    let filters = ExportFilters::new(args.excluded_namespaces.clone(), vec![], true);
    let req = ExportRequest::new(args.start_time, args.end_time, args.output_path.clone(), filters, args.format.clone().into());
    let endpoint = format!("{}/export", args.tracer_address);

    println!("exporting trace data");
    println!("start_ts = {}, end_ts = {}", args.start_time, args.end_time);
    println!("using filters:\n\texcluded_namespaces: {:?}\n\texcluded_labels: none", args.excluded_namespaces);
    println!("making request to {}", endpoint);

    let client = reqwest::Client::new();
    match client.post(endpoint).json(&req).send().await? {
        res if res.status().is_success() => {
            // If we got trace data back from the request, it means the tracer pod couldn't or
            // didn't want to write it (e.g., we asked to write to a local file); in the future we
            // might also try to write the data to the cloud provider storage as a fallback if it
            // didn't work from the tracer pod, so this will handle that case as well.
            let data = res.bytes().await?;
            if !data.is_empty() {
                let object_store = SkObjectStore::new(&args.output_path)?;
                object_store.put(data).await?;
            }
            println!("Trace data exported to {}", args.output_path);
        },
        res => bail!("Received {} response; could not export trace data:\n\n{}", res.status(), res.text().await?),
    };
    Ok(())
}
