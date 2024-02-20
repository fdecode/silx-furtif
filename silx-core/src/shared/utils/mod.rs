pub use crate::{
    structs::{
        archmod::archdata::{ ArchData, PinArchData, SlxData, },
        cells::ctrl_message::{ MsgFromServant, SendToMaster, ReplyToServant, }, 
        start_prod::{ StarterProducer, StarterProducerWithProcesses, StarterProducerWithFlow, },
    },
    traits::{
        procell::{ ProcessProducer, ProcessInstance, }, 
        filable::{ Filable, Filed, RecFiled, }, arch::ArchSized,
    },
    builder::{ ServantBuilder, ServantBuilderParameters, FiledStarter, Starter, },
};

#[macro_export]
/// Generate an empty process instance
/// * This is a macro helper for method `ProcessProducer::named_process(...)`
/// 
/// # Example:
///
/// ```text
/// let producer = ProcessProducer::new(&send_to_master);
/// let process_instance = produce!(producer);
/// ```
macro_rules! produce { ($prod: expr) => { 
    $prod.named_process()
} }

#[macro_export]
/// Generate an empty process instance with data
/// * This is a macro helper for method `ProcessProducer::named_process_with_data(...)`
/// 
/// # Example:
///
/// ```text
/// let producer = ProcessProducer::new(&send_to_master);
/// let process_instance = produce_data!(producer,"My data");
/// ```
macro_rules! produce_data { ($prod: expr, $data: expr) => { 
    $prod.named_process_with_data($data)
} }

#[macro_export]
/// Generate a process instance for a given future
/// * This is a macro helper for method `ProcessProducer::named_process_with_future(...)`
/// 
/// # Example:
///
/// ```text
/// let producer = ProcessProducer::new(&send_to_master);
/// let task = || { println!("task"); };
/// let my_future = async { println!("async task"); };
/// let process_instance = produce_future!(producer, {
///     task();
///     my_future.await;
/// });
/// ```
macro_rules! produce_future { ($prod: expr, $code: block) => {
        $prod.named_process_with_future(async move { $code std::future::pending::<()>().await })
} }

#[macro_export]
/// Generate a process instance for a given future with data
/// * This is a macro helper for method `ProcessProducer::named_process_with_future(...)`
/// * Typically, data may be anything whith shared handles moved to the future
/// 
/// # Example:
///
/// ```text
/// let producer = ProcessProducer::new(&send_to_master);
/// let task = || { println!("task"); };
/// let my_future = async { println!("async task"); };
/// let process_instance = produce_data_future!(producer,"My data", {
///     task();
///     my_future.await;
/// });
/// ```
macro_rules! produce_data_future { ($prod: expr, $data: expr, $code: block) => {
    $prod.named_process_with_data_future($data, async move { $code std::future::pending::<()>().await })
} }

pub use produce;
pub use produce_data;
pub use produce_future;
pub use produce_data_future;

#[macro_export]
/// Add a reply-to-query component of type 1 to the process producer
/// * This is a macro helper for method `ProcessProducer::add_reply1(...)`
/// 
/// # Example:
///
/// ```text
/// let mut producer = ProcessProducer::new(&send_to_master);
/// let hello_channel = "IncrementQuery".to_string();
/// produce_reply1!(producer, f64slx, hello_channel, data, {
///     let mut rnum: Pin<&mut f64slx> = data.arch_mut().unwrap();
///     *rnum += 1f64.slx();
/// })?;
/// ```
macro_rules! produce_reply1 { ($prod: expr, $typ: ty, $chan: expr, $data: ident, $code: block $(,)?) => {
    $prod.add_reply1::<$typ,_>(&$chan, move |$data| Box::pin(async move { $code }))
} }

#[macro_export]
/// Add a reply-to-query component of type 2 to the process producer
/// * This is a macro helper for method `ProcessProducer::add_reply2(...)`
/// * Options and restrictions: 
///    * the macro can move automatically variables which implement copy
///         * then use the command without bracketed list of variables, or with empty bracket, like in:  
///           `produce_reply2!(producer, U => V, channel_name, input, { ... code ...})`  
///           `produce_reply2!([], producer, U => V, channel_name, input, { ... code ...})`
///    * the macro cannot move automatically variables which implement clone but not copy
///         * then use bracketed list of variables like in:  
///           `produce_reply2!([var1, var2, ...], producer, U => V, channel_name, input, { ... code ...})`
///    * the macro cannot move variables which do not implement clone
/// 
/// # Examples:
/// ## Example with copiable data move:
///
/// ```text
/// let mut producer = ProcessProducer::new(&send_to_master);
/// let hello_channel = "HelloQuery".to_string();
/// let choice = true;
/// produce_reply2!(producer, String => String, hello_channel, data, {
///     let astr = data.archive_ref().unwrap();
///     let hs = if choice { format!("Hello {}", astr) } else  { format!("Good morning {}", astr) };
///     hs.arch_sized().unwrap()
/// })?;
/// ```
/// ## Example with clonable data move:
///
/// ```text
/// let mut producer = ProcessProducer::new(&send_to_master);
/// let hello_channel = "HelloQuery".to_string();
/// let specific_greeting_0 = "Good morning".to_string();
/// let specific_greeting_1 = "Good evening".to_string();
/// let choice = 1;
/// produce_reply2!([specific_greeting_0, specific_greeting_1], producer, String => String, hello_channel, data, {
///     let astr = data.archive_ref().unwrap();
///     let hs = match choice { 
///         0 => format!("{specific_greeting_0} {}", astr),
///         1 => format!("{specific_greeting_1} {}", astr),
///         _ => format!("Hello {}", astr),
///     };
///     hs.arch_sized().unwrap()
/// })?;
/// ```
macro_rules! produce_reply2 { 
    ($prod: expr, $intyp: ty => $outyp: ty, $chan: expr, $data: ident, $code: block $(,)?) => (
        produce_reply2!([], $prod, $intyp => $outyp, $chan, $data, $code)
    );
    ([$($I:ident),*], $prod: expr, $intyp: ty => $outyp: ty, $chan: expr, $data: ident, $code: block $(,)?) => (
        $prod.add_reply2::<$intyp,$outyp,_>(&$chan, move |$data| { // invoke `add_reply2`
            $(
                let $I= $I.clone(); // clone the moved data
            )*
            Box::pin(async move { $code })
        })
    );
}

#[macro_export]
/// Add a query component to the process producer, and get query sender and reply receiver
/// * This is a macro helper for method `ProcessProducer::add_query(...)`
/// 
/// # Example:
///
/// ```text
/// let mut producer = ProcessProducer::new(&send_to_master);
/// let hello_channel = "HelloQuery".to_string();
/// let capacity = Some(1);
/// let (sender,receiver) = produce_query!(producer, String => String, hello_channel, capacity)?;
/// ```
macro_rules! produce_query { ($prod: expr, $intyp: ty => $outyp: ty, $chan: expr, $capa: expr $(,)?) => {
    $prod.add_query::<$intyp,$outyp>(&$chan, $capa) 
} }

#[macro_export]
/// Add an emit component to the process producer, and get emit sender
/// * This is a macro helper for method `ProcessProducer::emit(...)`
/// 
/// # Example:
///
/// ```text
/// let mut producer = ProcessProducer::new(&send_to_master);
/// let wake_channel = "Wake".to_string();
/// let capacity = Some(1);
/// let sender = produce_emit!(producer, WakeSlx, wake_channel, capacity,)?;
/// ```
macro_rules! produce_emit { ($prod: expr, $typ: ty, $chan: expr, $capa: expr $(,)?) => { $prod.add_emit::<$typ>(&$chan, $capa) } }

#[macro_export]
/// Add an read component to the process producer, and get read receiver
/// * This is a macro helper for method `ProcessProducer::read(...)`
/// 
/// # Example:
///
/// ```text
/// let mut producer = ProcessProducer::new(&send_to_master);
/// let wake_channel = "Wake".to_string();
/// let capacity = Some(1);
/// let reader = produce_read!(producer, WakeSlx, wake_channel, capacity,)?;
/// ```
macro_rules! produce_read { ($prod: expr, $typ: ty, $chan: expr, $capa: expr $(,)?) => { $prod.add_read::<$typ>(&$chan, $capa) } }

#[macro_export]
/// Add read by reference component to the process producer
/// * This is a macro helper for method `ProcessProducer::add_ref_read(...)`
/// * Macro `produce_ref_read` is quite similar to macro `produce_reply1` except that data is sent as non-mutable reference
/// * Typical case of use of this component is to send a signal to an async process which will then react on this signal
/// 
/// # Example:
///
/// ```text
/// let mut producer = ProcessProducer::new(&send_to_master);
/// let warn_channel = "WarningChannel".to_string();
/// produce_ref_read!(producer, String, warn_channel, data, {
///     let astr = data.archive_ref().unwrap();
///     println!("WARNING ABOUT SOMETHING WRONG:\n {}", astr);
/// })?;
/// ```
macro_rules! produce_ref_read { ($prod: expr, $typ: ty, $chan: expr, $data: ident, $code: block $(,)?) => {
    $prod.add_ref_read::<$typ,_>(&$chan, move |$data| Box::pin(async move { $code }))
} }

pub use produce_reply1;
pub use produce_reply2;
pub use produce_query;
pub use produce_emit;
pub use produce_read;
pub use produce_ref_read;

/// Asynchroneous process which is permanently pending
pub async fn terminated() { std::future::pending::<()>().await }

