
use tokio::sync::broadcast;

/// WebSocket 服务器，处理 WebSocket 连接
#[derive(Debug)]
pub struct WebSocketServer {
    /// 广播通道，用于向所有连接的客户端发送消息
    pub broadcast_tx: broadcast::Sender<String>,
}

impl WebSocketServer {
    /// 创建一个新的 WebSocket 服务器
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            broadcast_tx,
        }
    }
    
    /// 广播消息给所有连接的客户端
    pub fn broadcast(&self, message: &str) {
        let _ = self.broadcast_tx.send(message.to_string());
    }
}
