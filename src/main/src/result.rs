use std::{convert::Infallible, fmt::Debug, os::unix::net::SocketAddr, path::PathBuf, sync::Arc};

use async_raft::{InitializeError, RaftError};
use camelpaste::paste;
use prost::{DecodeError, Message};
use qp2p::{EndpointError, SendError};
use thiserror::Error;
use tokio::task::JoinError;
use wasmedge_sdk::error::WasmEdgeError;
use zip_extract::ZipExtractError;

use crate::{
    general::{
        app::FnMeta,
        data::m_data_general::{DataItemIdx, DataSplitIdx, EachNodeSplit},
        network::{proto, rpc_model::HashValue},
    },
    sys::NodeID,
    util::TryUtf8VecU8,
};

pub type WSResult<T> = Result<T, WSError>;

#[derive(Debug)]
pub enum NotMatchNodeErr {
    NotRaft(String),
}

#[derive(Debug)]
pub enum WsNetworkLogicErr {
    DecodeError(DecodeError),
    MsgIdNotDispatchable(u32),
    InvaidNodeID(NodeID),
    TaskJoinError {
        err: tokio::task::JoinError
    },
}

#[derive(Debug)]
pub enum WsNetworkConnErr {
    EndPointError(EndpointError),
    SendError(SendError),
    ConnectionNotEstablished(NodeID),
    RPCTimout(NodeID),
    ConnectionExpired(NodeID),
}

#[derive(Debug)]
pub enum WsRpcErr {
    ConnectionNotEstablished(HashValue),
    RPCTimout(HashValue),
    InvalidMsgData { msg: Box<dyn Message> },
    UnknownPeer { peer: SocketAddr },
}

#[derive(Debug)]
pub enum WsIoErr {
    Io(std::io::Error),
    Io2(walkdir::Error),
    Zip(ZipExtractError),
    Zip2(zip::result::ZipError),
}

#[derive(Debug)]
pub enum WsRuntimeErr {
    TokioJoin { err: JoinError, context: String },
    ModulesRefOutofLifetime,
}

impl From<std::io::Error> for WSError {
    fn from(e: std::io::Error) -> Self {
        WSError::WsIoErr(WsIoErr::Io(e))
    }
}

impl From<walkdir::Error> for WSError {
    fn from(e: walkdir::Error) -> Self {
        WSError::WsIoErr(WsIoErr::Io2(e))
    }
}

impl From<zip::result::ZipError> for WSError {
    fn from(e: zip::result::ZipError) -> Self {
        WSError::WsIoErr(WsIoErr::Zip2(e))
    }
}

impl From<ZipExtractError> for WSError {
    fn from(e: ZipExtractError) -> Self {
        WSError::WsIoErr(WsIoErr::Zip(e))
    }
}

#[derive(Debug)]
pub enum WsRaftErr {
    InitializeError(async_raft::error::InitializeError),
    RaftError(RaftError),
}

#[derive(Debug)]
pub enum WsSerialErr {
    BincodeErr {
        err: Box<bincode::ErrorKind>,
        context: String,
    },
    AppMetaKvKeyIndexOutOfBound {
        app: String,
        func: String,
        index: usize,
        kvs_len: Option<usize>,
    },
}

#[derive(Error, Debug)]
pub enum WsFormatErr {
    #[error("KeyPatternFormatErr: {key_pattern}, '{{' '}}' should appear in pair, only '_' is allowed to appear as symbol")]
    KeyPatternFormatErr { key_pattern: String },
}

#[derive(Debug)]
pub enum WsPermissionErr {
    AccessKeyPermissionDenied {
        app: String,
        func: String,
        access_key: TryUtf8VecU8,
    },
}

#[derive(Debug)]
pub enum WsFuncError {
    WasmError(Box<wasmedge_sdk::error::WasmEdgeError>),
    AppNotFound {
        app: String,
    },
    InvalidAppMetaDataItem {
        app: String,
    },
    FuncNotFound {
        app: String,
        func: String,
    },
    InvalidHttpUrl(String),
    FuncHttpNotSupported {
        fname: String,
        fmeta: FnMeta,
    },
    FuncBackendHttpNotSupported {
        fname: String,
    },
    FuncHttpFail {
        app: String,
        func: String,
        http_err: reqwest::Error,
    },
    AppPackFailedZip(ZipExtractError),
    AppPackNoExe,
    AppPackExeName(String),
    AppPackConfReadInvalid(std::io::Error),
    AppPackConfDecodeErr(serde_yaml::Error),
    AppPackRemoveFailed(std::io::Error),
    AppPackTmp2NewFailed(std::io::Error),
    FuncSnapshotFailed {
        detail: String,
    },
    InstanceNotFound(String),
    InstanceTypeNotMatch {
        app: String,
        want: String,
    },
    CreateCracConfigFailed {
        path: String,
        err: std::io::Error,
    },
    InstanceJavaPidNotFound(String),
    InstanceProcessStartFailed(std::io::Error),
    InsranceVerifyFailed(String),
    UnsupportedAppType,
}

#[derive(Debug)]
pub enum WsDataError {
    InvalidDataType,
    DataSetNotFound {
        uniqueid: Vec<u8>,
    },
    GetDataFailed {
        unique_id: Vec<u8>,
        msg: String,
    },
    SetExpiredDataVersion {
        target_version: u64,
        cur_version: u64,
        data_id: String,
    },
    WriteDataRequireVersionErr {
        unique_id: Vec<u8>,
        err: Box<WSError>, // unique_id:String,
    },
    WriteDataSplitLenNotMatch {
        unique_id: Vec<u8>,
        expect: usize,
        actual: usize,
    },
    WriteDataFailed {
        unique_id: Vec<u8>,
        message: String,
    },
    KvDeserializeErr {
        unique_id: Vec<u8>,
        context: String,
    },
    KvGotWrongSplitCountAndIdx {
        unique_id: Vec<u8>,
        idx: Vec<DataItemIdx>,
    },
    KvEngineInnerError {
        inner: sled::Error,
        context: String,
    },
    KvEngineWriteLockReqiured {
        context: String,
    },
    KvEngineReadLockReqiured {
        context: String,
    },
    UnmatchedFileType {
        expect: proto::data_item::DataItemDispatch,
        actual: proto::data_item::DataItemDispatch,
        context: String,
    },
    FileMetadataErr {
        path: PathBuf,
        err: std::io::Error,
    },
    FileSeekErr {
        path: PathBuf,
        err: std::io::Error,
    },
    FileReadErr {
        path: PathBuf,
        err: std::io::Error,
    },
    FileOpenErr {
        path: PathBuf,
        err: std::io::Error,
    },
    FileRenameErr {
        from: PathBuf,
        to: PathBuf,
        err: std::io::Error,
    },
    FilePathParseErr {
        path: String,
        err: Infallible,
    },
    UnzipErr{
        path: PathBuf,
        err: ZipExtractError,
    },
    SplitRecoverMissing {
        unique_id: Vec<u8>,
        idx: DataItemIdx,
        missing: Vec<EachNodeSplit>,
    },
    SplitDataItemNotRawBytes {
        unique_id: Vec<u8>,
        splitidx: DataSplitIdx,
    },
    SplitDataItemNotFileData {
        unique_id: Vec<u8>,
        splitidx: DataSplitIdx,
    },
    SplitLenMismatch {
        unique_id: Vec<u8>,
        splitidx: DataSplitIdx,
        expect: usize,
        actual: usize,
    },
    SplitTaskFailed {
        request_id: proto::BatchRequestId,
        idx: DataSplitIdx,
    },
    BatchTransferTaskFailed {
        reason: String,
    },
    BatchTransferFailed {
        request_id: proto::BatchRequestId,
        reason: String,
    },
    BatchTransferNotFound {
        request_id: proto::BatchRequestId,
    },
    BatchTransferError {
        request_id: proto::BatchRequestId,
        msg: String,
    },
    UnknownCacheMapMode {
        mode: u16,
    },
    UnknownCacheTimeMode {
        mode: u16,
    },
    UnknownCachePosMode {
        mode: u16,
    },
    ItemIdxOutOfRange {
        wanted: DataItemIdx,
        len: u8,
    },
    ItemIdxEmpty,
    VersionMismatch {
        expected: u64,
        actual: u64,
    },
    SizeMismatch {
        expected: usize,  // 预期的数据大小
        actual: usize,    // 实际的数据大小
    },
    ReadDataFailed {
        path: PathBuf,     // 读取失败的文件路径
    },
    /// 数据分片任务错误
    DataSplitTaskError {
        msg: String,
    },
    /// 数据解码错误
    DataDecodeError {
        /// 错误原因
        reason: String,
        /// 数据类型（用于调试）
        data_type: String,
    },
}

#[derive(Error, Debug)]
pub enum WSError {
    #[error("ArcWrapper: {0:?}")]
    ArcWrapper(Arc<WSError>),

    #[error("Io error: {0:?}")]
    WsIoErr(WsIoErr),

    #[error("Network logic error: {0:?}")]
    WsNetworkLogicErr(WsNetworkLogicErr),

    #[error("Network connection error: {0:?}")]
    WsNetworkConnErr(WsNetworkConnErr),

    #[error("Raft error: {0:?}")]
    WsRaftErr(WsRaftErr),

    #[error("Permission error: {0:?}")]
    WsPermissionErr(WsPermissionErr),

    #[error("Serial error: {0:?}")]
    WsSerialErr(WsSerialErr),

    #[error("Format error: {0:?}")]
    WsFormatErr(WsFormatErr),

    #[error("Func error: {0:?}")]
    WsFuncError(WsFuncError),

    #[error("Rpc error: {0:?}")]
    WsRpcErr(WsRpcErr),

    #[error("Data error: {0:?}")]
    WsDataError(WsDataError),

    #[error("Runtime error: {0:?}")]
    WsRuntimeErr(WsRuntimeErr),

    #[error("Not Implemented")]
    NotImplemented,
}

impl From<WsNetworkLogicErr> for WSError {
    fn from(e: WsNetworkLogicErr) -> Self {
        WSError::WsNetworkLogicErr(e)
    }
}

impl From<WsNetworkConnErr> for WSError {
    fn from(e: WsNetworkConnErr) -> Self {
        WSError::WsNetworkConnErr(e)
    }
}

impl From<WsRaftErr> for WSError {
    fn from(e: WsRaftErr) -> Self {
        WSError::WsRaftErr(e)
    }
}

impl From<WsSerialErr> for WSError {
    fn from(e: WsSerialErr) -> Self {
        WSError::WsSerialErr(e)
    }
}

impl From<WsIoErr> for WSError {
    fn from(e: WsIoErr) -> Self {
        WSError::WsIoErr(e)
    }
}

impl From<WsPermissionErr> for WSError {
    fn from(e: WsPermissionErr) -> Self {
        WSError::WsPermissionErr(e)
    }
}

impl From<WsFuncError> for WSError {
    fn from(e: WsFuncError) -> Self {
        WSError::WsFuncError(e)
    }
}

impl From<WasmEdgeError> for WSError {
    fn from(e: WasmEdgeError) -> Self {
        WSError::WsFuncError(WsFuncError::WasmError(Box::new(e)))
    }
}

impl From<WsRpcErr> for WSError {
    fn from(e: WsRpcErr) -> Self {
        WSError::WsRpcErr(e)
    }
}

impl From<WsDataError> for WSError {
    fn from(value: WsDataError) -> Self {
        WSError::WsDataError(value)
    }
}

impl From<WsRuntimeErr> for WSError {
    fn from(e: WsRuntimeErr) -> Self {
        WSError::WsRuntimeErr(e)
    }
}

pub struct ErrCvt<T>(pub T);

macro_rules! impl_err_convertor {
    ($t:ty,$sub_t:ty,$sub_tt:ty) => {
        paste! {
            impl ErrCvt<$t> {
                pub fn [<to_ $sub_t:snake>](self) -> WSError {
                    WSError::$sub_t($sub_t::$sub_tt(self.0))
                }
            }
        }
    };
}

impl_err_convertor!(DecodeError, WsNetworkLogicErr, DecodeError);
impl_err_convertor!(EndpointError, WsNetworkConnErr, EndPointError);
impl_err_convertor!(SendError, WsNetworkConnErr, SendError);
impl_err_convertor!(InitializeError, WsRaftErr, InitializeError);
impl_err_convertor!(RaftError, WsRaftErr, RaftError);
impl_err_convertor!(std::io::Error, WsIoErr, Io);

pub trait WSResultExt :Sized {
    fn todo_handle(self, err_comment: &str) -> Self;
}

impl<T> WSResultExt for WSResult<T> {
    #[inline]
    fn todo_handle(self, err_comment: &str) -> Self {
        match &self {
            Ok(_ok) => {}
            Err(err) => {
                tracing::error!("{}, err: {:?}", err_comment, err);
            }
        }
        self
    }
}

// impl<T: Debug> WSResultExt for WSError {   WSError并没有泛型参数   去除特征约束    曾俊
impl WSResultExt for WSError {
    fn todo_handle(self, err_comment: &str) -> Self {
        tracing::error!("{}, err: {:?}", err_comment, self);
        self
    }
}

