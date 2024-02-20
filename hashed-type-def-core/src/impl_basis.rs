use std::io::Write;

use hashed_type_def_procmacro::{add_hashed_type_def, HashedTypeDef};

use crate::{ 
    impl_hashed_type_def_tuple, impl_hashed_type_def_fn, 
    proc_hashed_type_def_primitive, multi_hashed_type_def,
};

use super::{ add_hash_fnv1a, start_hash_fnv1a, HashedTypeDef, };

/// tag for HashedTypeDef implementation of std types
#[derive(HashedTypeDef)]
enum Standard {}

// implementation of HashedTypeDef for primitives
proc_hashed_type_def_primitive!(
    bool, char, str,
    f32, f64,
    i128, i16, i32, i64, i8, isize,
    u128, u16, u32, u64, u8, usize,
);

use std::{
    // Struct
    alloc::{ Layout, LayoutError, System, },
    any::TypeId,
    array::{ IntoIter, TryFromSliceError, },
    ascii::EscapeDefault, // // // //................
    backtrace::Backtrace,
    boxed::Box,
    cell::{ BorrowError, BorrowMutError, Cell, OnceCell, Ref, RefCell, RefMut, UnsafeCell, },
    char::{
        CharTryFromError, DecodeUtf16, DecodeUtf16Error,
        EscapeDebug,  // // // //................
        EscapeDefault as _Char_EscapeDefault,
        EscapeUnicode,  // // // //................
        ParseCharError, ToLowercase, ToUppercase, TryFromCharError,
    },
    cmp::Reverse,
    collections::{
        BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, TryReserveError, VecDeque,
        binary_heap::{ 
            Drain as _BinaryHeap_Drain, IntoIter as _BinaryHeap_IntoIter, 
            Iter as _BinaryHeap_Iter, PeekMut as _BinaryHeap_PeekMut,
        },
        btree_map::{
            IntoIter as _BTreeMap_IntoIter, IntoKeys as _BTreeMap_IntoKeys,
            IntoValues as _BTreeMap_IntoValues, Iter as _BTreeMap_Iter,
            IterMut as _BTreeMap_IterMut, Keys as _BTreeMap_Keys,
            OccupiedEntry as _BTreeMap_OccupiedEntry, Range as _BTreeMap_Range,
            RangeMut as _BTreeMap_RangeMut, VacantEntry as _BTreeMap_VacantEntry,
            Values as _BTreeMap_Values, ValuesMut as _BTreeMap_ValuesMut,
        },
        btree_set::{
            Difference as _BTreeSet_Difference, Intersection as _BTreeSet_Intersection,
            IntoIter as _BTreeSet_IntoIter, Iter as _BTreeSet_Iter,
            Range as _BTreeSet_Range, SymmetricDifference as _BTreeSet_SymmetricDifference,
            Union as _BTreeSet_Union,
        },
        hash_map::{
            DefaultHasher as _HashMap_DefaultHasher, Drain as _HashMap_Drain,
            IntoIter as _HashMap_IntoIter, IntoKeys as _HashMap_IntoKeys,
            IntoValues as _HashMap_IntoValues, Iter as _HashMap_Iter,
            IterMut as _HashMap_IterMut, Keys as _HashMap_Keys,
            OccupiedEntry as _HashMap_OccupiedEntry, RandomState as _HashMap_RandomState,
            VacantEntry as _HashMap_VacantEntry, Values as _HashMap_Values,
            ValuesMut as _HashMap_ValuesMut,
        },
        hash_set::{
            Difference as _HashSet_Difference, Drain as _HashSet_Drain,
            Intersection as _HashSet_Intersection, IntoIter as _HashSet_IntoIter,
            Iter as _HashSet_Iter, SymmetricDifference as _HashSet_SymmetricDifference,
            Union as _HashSet_Union,
        },
        linked_list::{
            IntoIter as _LinkedList_IntoIter, Iter as _LinkedList_Iter, IterMut as _LinkedList_IterMut,
        },
        vec_deque::{
            Drain as _VecDeque_Drain, IntoIter as _VecDeque_IntoIter,
            Iter as _VecDeque_Iter, IterMut as _VecDeque_IterMut,
        },
    },
    env::{
        Args, ArgsOs, JoinPathsError, SplitPaths, Vars, VarsOs,
    },
    ffi::{
        CStr, CString, FromBytesWithNulError, FromVecWithNulError,
        IntoStringError, NulError, OsStr, OsString,
    },
    fmt::{
        Arguments, DebugList, DebugMap, DebugSet, DebugStruct, DebugTuple, 
        Error,  // // // //................
        Formatter,
    },
    fs::{
        DirBuilder, DirEntry, File, FileType, Metadata, OpenOptions, Permissions, ReadDir,
    },
    future::{
        Pending, PollFn, Ready,
    },
    hash::BuildHasherDefault,
    io::{
        BufReader, BufWriter, 
        Bytes,  // // // //................
        Chain,  // // // //................
        Cursor,
        Empty,  // // // //................
        Error as _Io_Error, IntoInnerError,
        IoSlice, IoSliceMut, LineWriter,
        Lines,  // // // //................
        Repeat,  // // // //................
        Sink, 
        Split,  // // // //................
        Stderr, StderrLock, Stdin,
        StdinLock, Stdout, StdoutLock, 
        Take,  // // // //................
        WriterPanicked,
    },
    iter::{
        Chain as _Iter_Chain,
        Cloned, Copied, Cycle,
        Empty as _Iter_Empty,
        Enumerate, Filter, FilterMap, FlatMap, FromFn, Fuse, Inspect, Map, MapWhile, Once, OnceWith, Peekable, 
        Repeat as _Iter_Repeat,
        RepeatWith, Rev, Scan, Skip, SkipWhile, StepBy, Successors,
        Take as _Iter_Take,
        TakeWhile, Zip,
    },
    marker::{ PhantomData, PhantomPinned, },
    mem::{ Discriminant, ManuallyDrop, },
    net::{
        AddrParseError, Incoming, Ipv4Addr, Ipv6Addr, SocketAddrV4,
        SocketAddrV6, TcpListener, TcpStream, UdpSocket,
    },
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize,
        NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
        ParseFloatError, ParseIntError, TryFromIntError, Wrapping,
    },
    ops::{
        Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
    },
    option::{
        IntoIter as _Option_IntoIter, Iter as _Option_Iter, IterMut as _Option_IterMut,
    },
    panic::{
        AssertUnwindSafe, Location, PanicInfo,
    },
    path::{
        Ancestors, Components, Display, Iter as _Path_Iter, Path, PathBuf, PrefixComponent, StripPrefixError,
    },
    pin::Pin,
    process::{
        Child, ChildStderr, ChildStdin, ChildStdout, Command, CommandArgs,
        CommandEnvs, ExitCode, ExitStatus, Output, Stdio,
    },
    ptr::NonNull,
    rc::{ 
        Rc, 
        Weak,  // // // //................
    },
    result::{
        IntoIter as _Result_IntoIter, Iter as _Result_Iter, IterMut as _Result_IterMut,
    },
    slice::{
        Chunks, ChunksExact, ChunksExactMut, ChunksMut, EscapeAscii,
        Iter as _Slice_Iter, IterMut as _Slice_IterMut,
        RChunks, RChunksExact, RChunksExactMut, RChunksMut, RSplit, RSplitMut, RSplitN, RSplitNMut,
        Split as _Slice_Split,
        SplitInclusive, SplitInclusiveMut, SplitMut, SplitN, SplitNMut, Windows,
    },
    str::{
        Bytes as _Str_Bytes,
        CharIndices, Chars, EncodeUtf16, 
        EscapeDebug as _Str_EscapeDebug, 
        EscapeDefault as _Str_EscapeDefault,
        EscapeUnicode as _Str_EscapeUnicode,
        Lines as _Str_Lines,
        ParseBoolError, SplitAsciiWhitespace, SplitWhitespace, Utf8Error,
    },
    string::{
        Drain as _String_Drain,
        FromUtf16Error, FromUtf8Error, String,
    },
    sync::{
        Arc, Barrier, BarrierWaitResult, Condvar, Mutex, MutexGuard,
        Once as _Sync_Once,
        OnceLock, OnceState, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard, WaitTimeoutResult,
        Weak as _Sync_Weak,
        atomic::{
            AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize,
            AtomicPtr, AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize,
        },
        mpsc::{
            IntoIter as _Mpsc_IntoIter, Iter as _Mpsc_Iter,
            Receiver, RecvError, SendError, Sender, SyncSender, TryIter,
        },
    },
    task::{
        Context, RawWaker, RawWakerVTable, Waker,
    },
    thread::{
        AccessError, Builder, JoinHandle, LocalKey, Scope, ScopedJoinHandle, Thread, ThreadId,
    },
    time::{
        Duration, Instant, SystemTime, SystemTimeError, TryFromFloatSecsError,
    },
    vec::{
        Drain as _Vec_Drain,
        IntoIter as _Vec_IntoIter,
        Splice, Vec,
    },

    // Enum
    backtrace::BacktraceStatus,
    borrow::Cow,
    cmp::Ordering,  // // // //................
    collections::{
        btree_map::Entry as _BTreeMap_Entry,
        hash_map::Entry as _HashMap_Entry,
    },
    convert::Infallible,
    env::VarError,
    ffi::c_void,
    fmt::Alignment,
    io:: { ErrorKind, SeekFrom, },
    net::{ IpAddr, Shutdown, SocketAddr, },
    num::{ FpCategory, IntErrorKind, },
    ops::{ Bound, ControlFlow, },
    option::Option,
    path::{ Component, Prefix, },
    result::Result,
    sync::{
        TryLockError, atomic::Ordering as _Atomic_Ordering, mpsc::RecvTimeoutError,
        mpsc::{TryRecvError, TrySendError, },
    },
    task::Poll,
    // Union
    mem::MaybeUninit,
};

// implementation of HashedTypeDef for std structs, enums and union
multi_hashed_type_def! {
    // Struct
    struct Layout { std_alloc: Standard, };
    struct LayoutError { std_alloc: Standard, };
    struct System { std_alloc: Standard, };
    struct TypeId { std_any: Standard, };
    struct IntoIter<T, const N: usize> { std_array: Standard, };
    struct TryFromSliceError { std_array: Standard,};
    struct EscapeDefault { std_ascii: Standard, };
    struct Backtrace { std_backtrace: Standard, };
    struct Box<T: ?Sized,> { std_boxed: Standard, };
    struct BorrowError { std_cell: Standard, };
    struct BorrowMutError { std_cell: Standard, };
    struct Cell<T: ?Sized,> { std_cell: Standard, };
    struct OnceCell<T,> { std_cell: Standard, };
    struct Ref<'b, T: ?Sized + 'b,> { std_cell: Standard, };
    struct RefCell<T: ?Sized,> { std_cell: Standard, };
    struct RefMut<'b, T: ?Sized + 'b,> { std_cell: Standard, };
    struct UnsafeCell<T: ?Sized,> { std_cell: Standard, };
    struct CharTryFromError { std_char: Standard, };
    struct DecodeUtf16<I: Iterator<Item = u16,>,> { std_char: Standard, };
    struct DecodeUtf16Error { std_char: Standard, };
    struct EscapeDebug { std_char: Standard, };
    struct _Char_EscapeDefault { std_char: Standard, };
    struct EscapeUnicode { std_char: Standard, };
    struct ParseCharError { std_char: Standard, };
    struct ToLowercase { std_char: Standard, };
    struct ToUppercase { std_char: Standard, };
    struct TryFromCharError { std_char: Standard, };
    struct Reverse<T,> { std_cmp: Standard, };
    struct BTreeMap<K,V,> { std_collections: Standard, };
    struct BTreeSet<T,> { std_collections: Standard, };
    struct BinaryHeap<T,> { std_collections: Standard, };
    struct HashMap<K, V, S,> { std_collections: Standard, };
    struct HashSet<T, S,> { std_collections: Standard, };
    struct LinkedList<T,> { std_collections: Standard, };
    struct TryReserveError { std_collections: Standard, };
    struct VecDeque<T,> { std_collections: Standard, };
    struct _BinaryHeap_Drain<'a,T: 'a,> { std_collections_binary__heap: Standard, };
    struct _BinaryHeap_IntoIter<T,> { std_collections_binary__heap: Standard, };
    struct _BinaryHeap_Iter<'a, T: 'a,> { std_collections_binary__heap: Standard, };
    struct _BinaryHeap_PeekMut<'a, T: 'a + Ord,> { std_collections_binary__heap: Standard, };
    struct _BTreeMap_IntoIter<K, V,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_IntoKeys<K, V,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_IntoValues<K, V,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_Iter<'a, K: 'a, V: 'a,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_IterMut<'a, K: 'a, V: 'a,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_Keys<'a, K, V,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_OccupiedEntry<'a, K, V,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_Range<'a, K: 'a, V: 'a,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_RangeMut<'a, K: 'a, V: 'a,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_VacantEntry<'a, K, V,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_Values<'a, K, V,> { std_collections_btree__map: Standard, };
    struct _BTreeMap_ValuesMut<'a, K, V,> { std_collections_btree__map: Standard, };
    struct _BTreeSet_Difference<'a, T: 'a,> { std_collections_btree__set: Standard, };
    struct _BTreeSet_Intersection<'a, T: 'a,> { std_collections_btree__set: Standard, };
    struct _BTreeSet_IntoIter<T,> { std_collections_btree__set: Standard, };
    struct _BTreeSet_Iter<'a, T: 'a,> { std_collections_btree__set: Standard, };
    struct _BTreeSet_Range<'a, T: 'a,> { std_collections_btree__set: Standard, };
    struct _BTreeSet_SymmetricDifference<'a, T: 'a,> { std_collections_btree__set: Standard, };
    struct _BTreeSet_Union<'a, T: 'a,> { std_collections_btree__set: Standard, };
    struct _HashMap_DefaultHasher { std_collections_hash__map: Standard, };
    struct _HashMap_Drain<'a, K: 'a, V: 'a,> { std_collections_hash__map: Standard, };
    struct _HashMap_IntoIter<K, V,> { std_collections_hash__map: Standard, };
    struct _HashMap_IntoKeys<K, V,> { std_collections_hash__map: Standard, };
    struct _HashMap_IntoValues<K, V,> { std_collections_hash__map: Standard, };
    struct _HashMap_Iter<'a, K: 'a, V: 'a,> { std_collections_hash__map: Standard, };
    struct _HashMap_IterMut<'a, K: 'a, V: 'a,> { std_collections_hash__map: Standard, };
    struct _HashMap_Keys<'a, K: 'a, V: 'a,> { std_collections_hash__map: Standard, };
    struct _HashMap_OccupiedEntry<'a, K: 'a, V: 'a,> { std_collections_hash__map: Standard, };
    struct _HashMap_RandomState { std_collections_hash__map: Standard, };
    struct _HashMap_VacantEntry<'a, K: 'a, V: 'a,> { std_collections_hash__map: Standard, };
    struct _HashMap_Values<'a, K: 'a, V: 'a,> { std_collections_hash__map: Standard, };
    struct _HashMap_ValuesMut<'a, K: 'a, V: 'a,> { std_collections_hash__map: Standard, };
    struct _HashSet_Difference<'a, T: 'a, S: 'a,> { std_collections_hash__set: Standard, };
    struct _HashSet_Drain<'a, K: 'a,> { std_collections_hash__set: Standard, };
    struct _HashSet_Intersection<'a, T: 'a, S: 'a,> { std_collections_hash__set: Standard, };
    struct _HashSet_IntoIter<K,> { std_collections_hash__set: Standard, };
    struct _HashSet_Iter<'a, K: 'a,> { std_collections_hash__set: Standard, };
    struct _HashSet_SymmetricDifference<'a, T: 'a, S: 'a,> { std_collections_hash__set: Standard, };
    struct _HashSet_Union<'a, T: 'a, S: 'a,> { std_collections_hash__set: Standard, };
    struct _LinkedList_IntoIter<T,> { std_collections_linked__list: Standard, };
    struct _LinkedList_Iter<'a, T: 'a,> { std_collections_linked__list: Standard, };
    struct _LinkedList_IterMut<'a, T: 'a,> { std_collections_linked__list: Standard, };    
    struct _VecDeque_Drain<'a, T: 'a,> { std_collections_vec__deque: Standard, };
    struct _VecDeque_IntoIter<T,> { std_collections_vec__deque: Standard, };
    struct _VecDeque_Iter<'a, T: 'a,> { std_collections_vec__deque: Standard, };
    struct _VecDeque_IterMut<'a, T: 'a,> { std_collections_vec__deque: Standard, };
    struct Args { std_env: Standard, };
    struct ArgsOs { std_env: Standard, };
    struct JoinPathsError { std_env: Standard, };
    struct SplitPaths<'a,> { std_env: Standard, };
    struct Vars { std_env: Standard, };
    struct VarsOs { std_env: Standard, };
    struct CStr { std_ffi: Standard, };
    struct CString { std_ffi: Standard, };
    struct FromBytesWithNulError { std_ffi: Standard, };
    struct FromVecWithNulError { std_ffi: Standard, };
    struct IntoStringError { std_ffi: Standard, };
    struct NulError { std_ffi: Standard, };
    struct OsStr { std_ffi: Standard, };
    struct OsString { std_ffi: Standard, };
    struct Arguments<'a,> { std_fmt: Standard, };
    struct DebugList<'a, 'b: 'a,> { std_fmt: Standard, };
    struct DebugMap<'a, 'b: 'a,> { std_fmt: Standard, };
    struct DebugSet<'a, 'b: 'a,> { std_fmt: Standard, };
    struct DebugStruct<'a, 'b: 'a,> { std_fmt: Standard, };
    struct DebugTuple<'a, 'b: 'a,> { std_fmt: Standard, };
    struct Error { std_fmt: Standard, };
    struct Formatter<'a,> { std_fmt: Standard, };
    struct DirBuilder { std_fs: Standard, };
    struct DirEntry { std_fs: Standard, };
    struct File { std_fs: Standard, };
    struct FileType { std_fs: Standard, };
    struct Metadata { std_fs: Standard, };
    struct OpenOptions { std_fs: Standard, };
    struct Permissions { std_fs: Standard, };
    struct ReadDir { std_fs: Standard, };
    struct Pending<T,> { std_future: Standard, };
    struct PollFn<F,> { std_future: Standard, };
    struct Ready<T,> { std_future: Standard, };
    struct BuildHasherDefault<H,> { std_hash: Standard, };
    struct BufReader<R,> { std_io: Standard, };
    struct BufWriter<W: Write,> { std_io: Standard, };
    struct Bytes<R,> { std_io: Standard, };
    struct Chain<T, U,> { std_io: Standard, };
    struct Cursor<T,> { std_io: Standard, };
    struct Empty { std_io: Standard, };
    struct _Io_Error { std_io: Standard, };
    struct IntoInnerError<W,> { std_io: Standard, };
    struct IoSlice<'a,> { std_io: Standard, };
    struct IoSliceMut<'a,> { std_io: Standard, };
    struct LineWriter<W: Write,> { std_io: Standard, };
    struct Lines<B,> { std_io: Standard, };
    struct Repeat { std_io: Standard, };
    struct Sink { std_io: Standard, };
    struct Split<B,> { std_io: Standard, };
    struct Stderr { std_io: Standard, };
    struct StderrLock<'a,> { std_io: Standard, };
    struct Stdin { std_io: Standard, };
    struct StdinLock<'a,> { std_io: Standard, };
    struct Stdout { std_io: Standard, };
    struct StdoutLock<'a,> { std_io: Standard, };
    struct Take<T,> { std_io: Standard, };
    struct WriterPanicked { std_io: Standard, };
    struct _Iter_Chain<A,B,> { std_iter: Standard, };
    struct Cloned<I,> { std_iter: Standard, };
    struct Copied<I,> { std_iter: Standard, };
    struct Cycle<I,> { std_iter: Standard, };
    struct _Iter_Empty<T,> { std_iter: Standard, };
    struct Enumerate<I,> { std_iter: Standard, };
    struct Filter<I,P,> { std_iter: Standard, };
    struct FilterMap<I,F,> { std_iter: Standard, };
    struct FlatMap<I, U: IntoIterator, F,> { std_iter: Standard, };
    struct FromFn<F,> { std_iter: Standard, };
    struct Fuse<I,> { std_iter: Standard, };
    struct Inspect<I, F,> { std_iter: Standard, };
    struct Map<I, F,> { std_iter: Standard, };
    struct MapWhile<I,P,> { std_iter: Standard, };
    struct Once<T,> { std_iter: Standard, };
    struct OnceWith<F,> { std_iter: Standard, };
    struct Peekable<I: Iterator,> { std_iter: Standard, };
    struct _Iter_Repeat<A,> { std_iter: Standard, };
    struct RepeatWith<F,> { std_iter: Standard, };
    struct Rev<T,> { std_iter: Standard, };
    struct Scan<I, St, F,> { std_iter: Standard, };
    struct Skip<I,> { std_iter: Standard, };
    struct SkipWhile<I,P,> { std_iter: Standard, };
    struct StepBy<I,> { std_iter: Standard, };
    struct Successors<T,F,> { std_iter: Standard, };
    struct _Iter_Take<I,> { std_iter: Standard, };
    struct TakeWhile<I,P,> { std_iter: Standard, };
    struct Zip<A,B,> { std_iter: Standard, };
    struct PhantomData<T: ?Sized,> { std_marker: Standard, };
    struct PhantomPinned { std_marker: Standard, };
    struct Discriminant<T,> { std_mem: Standard, };
    struct ManuallyDrop<T: ?Sized,> { std_mem: Standard, };
    struct AddrParseError { std_net: Standard, };
    struct Incoming<'a,> { std_net: Standard, };
    struct Ipv4Addr { std_net: Standard, };
    struct Ipv6Addr { std_net: Standard, };
    struct SocketAddrV4 { std_net: Standard, };
    struct SocketAddrV6 { std_net: Standard, };
    struct TcpListener { std_net: Standard, };
    struct TcpStream { std_net: Standard, };
    struct UdpSocket { std_net: Standard, };
    struct NonZeroI128 { std_num: Standard, };
    struct NonZeroI16 { std_num: Standard, };
    struct NonZeroI32 { std_num: Standard, };
    struct NonZeroI64 { std_num: Standard, };
    struct NonZeroI8 { std_num: Standard, };
    struct NonZeroIsize { std_num: Standard, };
    struct NonZeroU128 { std_num: Standard, };
    struct NonZeroU16 { std_num: Standard, };
    struct NonZeroU32 { std_num: Standard, };
    struct NonZeroU64 { std_num: Standard, };
    struct NonZeroU8 { std_num: Standard, };
    struct NonZeroUsize { std_num: Standard, };
    struct ParseFloatError { std_num: Standard, };
    struct ParseIntError { std_num: Standard, };
    struct TryFromIntError { std_num: Standard, };
    struct Wrapping<T,> { std_num: Standard, };
    struct Range<Idx,> { std_ops: Standard, };
    struct RangeFrom<Idx,> { std_ops: Standard, };
    struct RangeFull { std_ops: Standard, };
    struct RangeInclusive<Idx,> { std_ops: Standard, };
    struct RangeTo<Idx,> { std_ops: Standard, };
    struct RangeToInclusive<Idx,> { std_ops: Standard, };
    struct _Option_IntoIter<A,> { std_option: Standard, };
    struct _Option_Iter<'a, A: 'a,> { std_option: Standard, };
    struct _Option_IterMut<'a, A: 'a,> { std_option: Standard, };
    struct AssertUnwindSafe<T,> { std_panic: Standard, };
    struct Location<'a,> { std_panic: Standard, };
    struct PanicInfo<'a,> { std_panic: Standard, };
    struct Ancestors<'a,> { std_path: Standard, };
    struct Components<'a,> { std_path: Standard, };
    struct Display<'a,> { std_path: Standard, };
    struct _Path_Iter<'a,> { std_path: Standard, };
    struct Path { std_path: Standard, };
    struct PathBuf { std_path: Standard, };
    struct PrefixComponent<'a,> { std_path: Standard, };
    struct StripPrefixError { std_path: Standard, };
    struct Pin<P,> { std_pin: Standard, };
    struct Child { std_process: Standard, };
    struct ChildStderr { std_process: Standard, };
    struct ChildStdin { std_process: Standard, };
    struct ChildStdout { std_process: Standard, };
    struct Command { std_process: Standard, };
    struct CommandArgs<'a,> { std_process: Standard, };
    struct CommandEnvs<'a,> { std_process: Standard, };
    struct ExitCode { std_process: Standard, };
    struct ExitStatus { std_process: Standard, };
    struct Output { std_process: Standard, };
    struct Stdio { std_process: Standard, };
    struct NonNull<T: ?Sized,> { std_ptr: Standard, };
    struct Rc<T: ?Sized,> { std_rc: Standard, };
    struct Weak<T: ?Sized,> { std_ptr: Standard, };
    struct _Result_IntoIter<T,> { std_result: Standard, };
    struct _Result_Iter<'a, T: 'a,> { std_result: Standard, };
    struct _Result_IterMut<'a, T: 'a,> { std_result: Standard, };
    struct Chunks<'a, T: 'a,> { std_slice: Standard, };
    struct ChunksExact<'a, T: 'a,> { std_slice: Standard, };
    struct ChunksExactMut<'a, T: 'a,> { std_slice: Standard, };
    struct ChunksMut<'a, T: 'a,> { std_slice: Standard, };
    struct EscapeAscii<'a,> { std_slice: Standard, };
    struct _Slice_Iter<'a, T: 'a,> { std_slice: Standard, };
    struct _Slice_IterMut<'a, T: 'a,> { std_slice: Standard, };
    struct RChunks<'a, T: 'a,> { std_slice: Standard, };
    struct RChunksExact<'a, T: 'a,> { std_slice: Standard, };
    struct RChunksExactMut<'a, T: 'a,> { std_slice: Standard, };
    struct RChunksMut<'a, T: 'a,> { std_slice: Standard, };
    struct RSplit<'a, T: 'a, P: FnMut(&T) -> bool,> { std_slice: Standard, };
    struct RSplitMut<'a, T: 'a, P: FnMut(&T) -> bool,> { std_slice: Standard, };
    struct RSplitN<'a, T: 'a, P: FnMut(&T) -> bool,> { std_slice: Standard, };
    struct RSplitNMut<'a, T: 'a, P: FnMut(&T) -> bool,> { std_slice: Standard, };
    struct _Slice_Split<'a, T: 'a, P: FnMut(&T) -> bool,> { std_slice: Standard, };
    struct SplitInclusive<'a, T: 'a, P: FnMut(&T) -> bool,> { std_slice: Standard, };
    struct SplitInclusiveMut<'a, T: 'a, P: FnMut(&T) -> bool,> { std_slice: Standard, };
    struct SplitMut<'a, T: 'a, P: FnMut(&T) -> bool,> { std_slice: Standard, };
    struct SplitN<'a, T: 'a, P: FnMut(&T) -> bool,> { std_slice: Standard, };
    struct SplitNMut<'a, T: 'a, P: FnMut(&T) -> bool,> { std_slice: Standard, };
    struct Windows<'a, T: 'a,> { std_slice: Standard, };
    struct _Str_Bytes<'a,> { std_str: Standard, };
    struct CharIndices<'a,> { std_str: Standard, };
    struct Chars<'a,> { std_str: Standard, };
    struct EncodeUtf16<'a,> { std_str: Standard, };
    struct _Str_EscapeDebug<'a,> { std_str: Standard, };
    struct _Str_EscapeDefault<'a,> { std_str: Standard, };
    struct _Str_EscapeUnicode<'a,> { std_str: Standard, };
    struct _Str_Lines<'a,> { std_str: Standard, };
    struct ParseBoolError { std_str: Standard, };
    struct SplitAsciiWhitespace<'a,> { std_str: Standard, };
    struct SplitWhitespace<'a,> { std_str: Standard, };
    struct Utf8Error { std_str: Standard, };
    struct _String_Drain<'a,> { std_string: Standard, };
    struct FromUtf16Error { std_string: Standard, };
    struct FromUtf8Error { std_string: Standard, };
    struct String { std_string: Standard, };

    struct Arc<T: ?Sized,> { std_sync: Standard, };
    struct Barrier { std_sync: Standard, };
    struct BarrierWaitResult { std_sync: Standard, };
    struct Condvar { std_sync: Standard, };
    struct Mutex<T: ?Sized,> { std_sync: Standard, };
    struct MutexGuard<'a, T: ?Sized + 'a,> { std_sync: Standard, };
    struct _Sync_Once { std_sync: Standard, };
    struct OnceLock<T,> { std_sync: Standard, };
    struct OnceState { std_sync: Standard, };
    struct PoisonError<T,> { std_sync: Standard, };
    struct RwLock<T: ?Sized,> { std_sync: Standard, };
    struct RwLockReadGuard<'a, T: ?Sized + 'a,> { std_sync: Standard, };
    struct RwLockWriteGuard<'a, T: ?Sized + 'a,> { std_sync: Standard, };
    struct WaitTimeoutResult { std_sync: Standard, };
    struct _Sync_Weak<T: ?Sized,> { std_sync: Standard, };
    struct AtomicBool { std_sync_atomic: Standard, };
    struct AtomicI16 { std_sync_atomic: Standard, };
    struct AtomicI32 { std_sync_atomic: Standard, };
    struct AtomicI64 { std_sync_atomic: Standard, };
    struct AtomicI8 { std_sync_atomic: Standard, };
    struct AtomicIsize { std_sync_atomic: Standard, };
    struct AtomicPtr<T,> { std_sync_atomic: Standard, };
    struct AtomicU16 { std_sync_atomic: Standard, };
    struct AtomicU32 { std_sync_atomic: Standard, };
    struct AtomicU64 { std_sync_atomic: Standard, };
    struct AtomicU8 { std_sync_atomic: Standard, };
    struct AtomicUsize { std_sync_atomic: Standard, };
    struct _Mpsc_IntoIter<T,> { std_sync_mpsc: Standard, };
    struct _Mpsc_Iter<'a, T: 'a,> { std_sync_mpsc: Standard, };
    struct Receiver<T,> { std_sync_mpsc: Standard, };
    struct RecvError { std_sync_mpsc: Standard, };
    struct SendError<T,> { std_sync_mpsc: Standard, };
    struct Sender<T,> { std_sync_mpsc: Standard, };
    struct SyncSender<T,> { std_sync_mpsc: Standard, };
    struct TryIter<'a, T: 'a,> { std_sync_mpsc: Standard, };
    struct Context<'a,> { std_task: Standard, };
    struct RawWaker { std_task: Standard, };
    struct RawWakerVTable { std_task: Standard, };
    struct Waker { std_task: Standard, };
    struct AccessError { std_thread: Standard, };
    struct Builder { std_thread: Standard, };
    struct JoinHandle<T,> { std_thread: Standard, };
    struct LocalKey<T: 'static,> { std_thread: Standard, };
    struct Scope<'scope, 'env: 'scope,> { std_thread: Standard, };
    struct ScopedJoinHandle<'scope, T,> { std_thread: Standard, };
    struct Thread { std_thread: Standard, };
    struct ThreadId { std_thread: Standard, };
    struct Duration { std_time: Standard, };
    struct Instant { std_time: Standard, };
    struct SystemTime { std_time: Standard, };
    struct SystemTimeError { std_time: Standard, };
    struct TryFromFloatSecsError { std_time: Standard, };
    struct _Vec_Drain<'a, T: 'a,> { std_vec: Standard, };
    struct _Vec_IntoIter<T,> { std_vec: Standard, };
    struct Splice<'a, I: 'a + Iterator,> { std_vec: Standard, };
    struct Vec<T,> { std_vec: Standard, };
    // Enum
    enum BacktraceStatus { StdBacktrace(Standard) };
    enum Cow<'a, B: ?Sized + 'a + ToOwned,> { StdBorrow(Standard) };
    enum Ordering { StdCmp(Standard) };
    enum _BTreeMap_Entry<'a, K: 'a, V: 'a,> { StdCollectionsBtreeMap(Standard) };
    enum _HashMap_Entry<'a, K: 'a, V: 'a,> { StdCollectionsHashMap(Standard) };
    enum Infallible { StdConvert(Standard) };
    enum VarError { StdEnv(Standard) };
    enum c_void { StdFfi(Standard) };
    enum Alignment { StdFmt(Standard) };
    enum ErrorKind { StdIo(Standard) };
    enum SeekFrom { StdIo(Standard) };
    enum IpAddr { StdNet(Standard) };
    enum Shutdown { StdNet(Standard) };
    enum SocketAddr { StdNet(Standard) };
    enum FpCategory { StdNum(Standard) };
    enum IntErrorKind { StdNum(Standard) };
    enum Bound<T,> { StdOps(Standard) };
    enum ControlFlow<B, C,> { StdOps(Standard) };
    enum Option<T,> { StdOption(Standard) };
    enum Component<'a,> { StdPath(Standard) };
    enum Prefix<'a,> { StdPath(Standard) };
    enum Result<T,E,> { StdResult(Standard) };
    enum TryLockError<T,> { StdSync(Standard) };
    enum _Atomic_Ordering { StdSyncAtomic(Standard) };
    enum RecvTimeoutError { StdSyncMpsc(Standard) };
    enum TryRecvError { StdSyncMpsc(Standard) };
    enum TrySendError<T,> { StdSyncMpsc(Standard) };
    enum Poll<T,> { StdTask(Standard) };
    // Union
    union MaybeUninit<T,> { std_mem: Standard };
}

// implementation of HashedTypeDef for tuples
impl_hashed_type_def_tuple!(());
impl_hashed_type_def_tuple!((T0,));
impl_hashed_type_def_tuple!((T0, T1,));
impl_hashed_type_def_tuple!((T0, T1, T2,));
impl_hashed_type_def_tuple!((T0, T1, T2, T3,));
impl_hashed_type_def_tuple!((T0, T1, T2, T3, T4,));
impl_hashed_type_def_tuple!((T0, T1, T2, T3, T4, T5,));
impl_hashed_type_def_tuple!((T0, T1, T2, T3, T4, T5, T6,));
impl_hashed_type_def_tuple!((T0, T1, T2, T3, T4, T5, T6, T7,));
impl_hashed_type_def_tuple!((T0, T1, T2, T3, T4, T5, T6, T7, T8,));
impl_hashed_type_def_tuple!((T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,));
impl_hashed_type_def_tuple!((T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, TA,));
impl_hashed_type_def_tuple!((T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, TA, TB,));

// implementation of HashedTypeDef for functions
impl_hashed_type_def_fn!(fn() -> Ret);
impl_hashed_type_def_fn!(fn(T0,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1, T2,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1, T2, T3,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1, T2, T3, T4,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1, T2, T3, T4, T5,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1, T2, T3, T4, T5, T6,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1, T2, T3, T4, T5, T6, T7,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1, T2, T3, T4, T5, T6, T7, T8,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, TA,) -> Ret);
impl_hashed_type_def_fn!(fn(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, TA, TB,) -> Ret);

// implementation of HashedTypeDef for arrays
impl<T: HashedTypeDef, const N: usize> HashedTypeDef for [T;N] {
    const TYPE_HASH_NATIVE: u128 = {
        let mut hash = start_hash_fnv1a(b"<array>");
        hash = add_hash_fnv1a(&T::TYPE_HASH_NATIVE.to_le_bytes(),hash);
        hash = add_hash_fnv1a(&N.to_le_bytes(),hash);
        hash
    };
}

// implementation of HashedTypeDef for constant pointers
impl<T: HashedTypeDef + ?Sized> HashedTypeDef for *const T {
    const TYPE_HASH_NATIVE: u128 = {
        let mut hash = start_hash_fnv1a(b"<ptr const>");
        hash = add_hash_fnv1a(&T::TYPE_HASH_NATIVE.to_le_bytes(),hash);
        hash
    };
}

// implementation of HashedTypeDef for mutable pointers
impl<T: HashedTypeDef + ?Sized> HashedTypeDef for *mut T {
    const TYPE_HASH_NATIVE: u128 = {
        let mut hash = start_hash_fnv1a(b"<ptr mut>");
        hash = add_hash_fnv1a(&T::TYPE_HASH_NATIVE.to_le_bytes(),hash);
        hash
    };
}

// implementation of HashedTypeDef for slices
impl<T: HashedTypeDef> HashedTypeDef for [T] {
    const TYPE_HASH_NATIVE: u128 = {
        let mut hash = start_hash_fnv1a(b"<slice>");
        hash = add_hash_fnv1a(&T::TYPE_HASH_NATIVE.to_le_bytes(),hash);
        hash
    };
}

// implementation of HashedTypeDef for references
impl<'a, T: ?Sized + HashedTypeDef> HashedTypeDef for &'a T {
    const TYPE_HASH_NATIVE: u128 = {
        let mut hash = start_hash_fnv1a(b"<ref>");
        hash = add_hash_fnv1a(&T::TYPE_HASH_NATIVE.to_le_bytes(),hash);
        hash
    };
}

// implementation of HashedTypeDef for mutable references
impl<'a, T: ?Sized + HashedTypeDef> HashedTypeDef for &'a mut T {
    const TYPE_HASH_NATIVE: u128 = {
        let mut hash = start_hash_fnv1a(b"<ref mut>");
        hash = add_hash_fnv1a(&T::TYPE_HASH_NATIVE.to_le_bytes(),hash);
        hash
    };
}

