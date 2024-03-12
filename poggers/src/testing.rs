use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_tree::HierarchicalLayer;

pub(crate) fn init_tracing() {
    let layer = HierarchicalLayer::default()
        .with_writer(std::io::stdout)
        .with_indent_lines(true)
        .with_indent_amount(2)
        // .with_thread_names(true)
        // .with_thread_ids(true)
        .with_targets(true);
    let subscriber = Registry::default().with(layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
