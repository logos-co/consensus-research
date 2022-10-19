use tracing_core::Subscriber;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::Layer;

pub const CLARO_TARGET_TAG: &str = "CLARO_TARGET";

pub fn claro_tracing_layer_with_writer<W, S>(writer: W, filter_tag: &'static str) -> impl Layer<S>
where
    W: for<'writer> MakeWriter<'writer> + 'static,
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer()
        .with_writer(writer)
        .json()
        .with_filter(filter_fn(move |metadata| metadata.target() == filter_tag))
}
