// JLiverTool Plugin Preload Script
// This script is injected into plugin webviews to provide the jliverAPI

(function() {
    'use strict';

    // Get WebSocket port from URL parameters
    const urlParams = new URLSearchParams(window.location.search);
    const wsPort = urlParams.get('ws_port');

    if (!wsPort) {
        console.error('JLiverTool: No WebSocket port provided');
        return;
    }

    // WebSocket connection
    let ws = null;
    let reconnectTimer = null;
    let requestId = 0;
    const pendingRequests = new Map();
    const eventCallbacks = new Map();

    // Connect to WebSocket server
    function connect() {
        if (ws && ws.readyState === WebSocket.OPEN) {
            return;
        }

        ws = new WebSocket(`ws://127.0.0.1:${wsPort}`);

        ws.onopen = function() {
            console.log('JLiverTool: Connected to plugin server');
            if (reconnectTimer) {
                clearTimeout(reconnectTimer);
                reconnectTimer = null;
            }
        };

        ws.onmessage = function(event) {
            try {
                const msg = JSON.parse(event.data);
                handleMessage(msg);
            } catch (e) {
                console.error('JLiverTool: Failed to parse message', e);
            }
        };

        ws.onclose = function() {
            console.log('JLiverTool: Disconnected from plugin server');
            scheduleReconnect();
        };

        ws.onerror = function(error) {
            console.error('JLiverTool: WebSocket error', error);
        };
    }

    function scheduleReconnect() {
        if (!reconnectTimer) {
            reconnectTimer = setTimeout(function() {
                reconnectTimer = null;
                connect();
            }, 3000);
        }
    }

    function handleMessage(msg) {
        switch (msg.type) {
            case 'Welcome':
                console.log('JLiverTool: Server welcome, port:', msg.port);
                break;

            case 'Event':
                // Dispatch event to registered callbacks
                const eventType = msg.Event.type.toLowerCase();
                const callbacks = eventCallbacks.get(eventType) || [];
                const allCallbacks = eventCallbacks.get('*') || [];

                [...callbacks, ...allCallbacks].forEach(cb => {
                    try {
                        cb(msg.Event);
                    } catch (e) {
                        console.error('JLiverTool: Event callback error', e);
                    }
                });
                break;

            case 'Response':
                // Resolve pending request
                const resolver = pendingRequests.get(msg.id);
                if (resolver) {
                    pendingRequests.delete(msg.id);
                    resolver.resolve(msg.data);
                }
                break;

            case 'Error':
                // Reject pending request or log error
                if (msg.id) {
                    const resolver = pendingRequests.get(msg.id);
                    if (resolver) {
                        pendingRequests.delete(msg.id);
                        resolver.reject(new Error(msg.message));
                    }
                } else {
                    console.error('JLiverTool: Server error', msg.message);
                }
                break;
        }
    }

    function sendMessage(msg) {
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify(msg));
        } else {
            console.warn('JLiverTool: WebSocket not connected');
        }
    }

    function request(method, params) {
        return new Promise((resolve, reject) => {
            const id = String(++requestId);
            pendingRequests.set(id, { resolve, reject });

            sendMessage({
                type: 'Request',
                id: id,
                method: method,
                params: params || {}
            });

            // Timeout after 30 seconds
            setTimeout(() => {
                if (pendingRequests.has(id)) {
                    pendingRequests.delete(id);
                    reject(new Error('Request timeout'));
                }
            }, 30000);
        });
    }

    // Public API
    window.jliverAPI = {
        // Register event callback
        // Channels: 'new_danmu', 'new_gift', 'new_guard', 'new_superchat',
        //           'new_interact', 'update_room', 'update_online', 'live_start', 'live_end'
        // Use '*' to receive all events
        register: function(channel, callback) {
            const channelLower = channel.toLowerCase();
            if (!eventCallbacks.has(channelLower)) {
                eventCallbacks.set(channelLower, []);
            }
            eventCallbacks.get(channelLower).push(callback);

            // Subscribe on server
            sendMessage({
                type: 'Subscribe',
                channels: [channelLower]
            });

            // Return unregister function
            return function unregister() {
                const callbacks = eventCallbacks.get(channelLower);
                if (callbacks) {
                    const idx = callbacks.indexOf(callback);
                    if (idx >= 0) {
                        callbacks.splice(idx, 1);
                    }
                }
            };
        },

        // Utility functions
        util: {
            openUrl: function(url) {
                return request('openUrl', { url: url });
            },
            getServerInfo: function() {
                return request('getServerInfo', {});
            }
        },

        // Connection status
        isConnected: function() {
            return ws && ws.readyState === WebSocket.OPEN;
        },

        // Reconnect manually
        reconnect: function() {
            if (ws) {
                ws.close();
            }
            connect();
        }
    };

    // Start connection
    connect();

    console.log('JLiverTool Plugin API loaded');
})();
