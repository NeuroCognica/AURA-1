const WebSocket = require('ws');
const url = 'ws://127.0.0.1:8080/ws/ai';
const ws = new WebSocket(url);
ws.on('open', () => { console.log('[ws_test] connected'); });
ws.on('message', (data) => {
  console.log('[ws_test] message:', data.toString());
});
ws.on('close', () => { console.log('[ws_test] closed'); });
ws.on('error', (e) => { console.error('[ws_test] error', e); });
