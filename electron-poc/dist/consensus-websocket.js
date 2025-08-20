"use strict";
/**
 * WebSocket streaming client for Hive Consensus
 * Handles real-time consensus pipeline updates
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.formatCost = exports.formatTokens = exports.STAGE_DISPLAY_NAMES = exports.ConsensusWebSocket = void 0;
class ConsensusWebSocket {
    constructor(url, callbacks) {
        this.ws = null;
        this.reconnectTimer = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.isManuallyDisconnected = false;
        this.pingInterval = null;
        this.connected = false;
        this.listenersSetup = false;
        this.url = url;
        this.callbacks = callbacks;
        this.setupIPCListeners();
    }
    setupIPCListeners() {
        // Only set up listeners once to avoid duplication
        if (this.listenersSetup) {
            return;
        }
        this.listenersSetup = true;
        // Set up message listeners once
        window.websocketAPI.onMessage((data) => {
            try {
                const message = JSON.parse(data);
                this.handleMessage(message);
            }
            catch (error) {
                console.error('Failed to parse WebSocket message:', error);
            }
        });
        window.websocketAPI.onError((error) => {
            var _a, _b, _c, _d;
            console.error('WebSocket error via IPC:', error);
            (_b = (_a = this.callbacks).onError) === null || _b === void 0 ? void 0 : _b.call(_a, 'WebSocket connection error');
            this.connected = false;
            (_d = (_c = this.callbacks).onConnectionStateChange) === null || _d === void 0 ? void 0 : _d.call(_c, false);
        });
        window.websocketAPI.onClose(() => {
            var _a, _b;
            console.log('WebSocket closed via IPC');
            this.connected = false;
            (_b = (_a = this.callbacks).onConnectionStateChange) === null || _b === void 0 ? void 0 : _b.call(_a, false);
            this.scheduleReconnect();
        });
    }
    connectViaIPC() {
        var _a, _b, _c, _d;
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Connect via IPC
                const result = yield window.websocketAPI.connect(this.url);
                if (result.connected) {
                    console.log('WebSocket connected successfully via IPC');
                    this.connected = true;
                    this.reconnectAttempts = 0;
                    (_b = (_a = this.callbacks).onConnectionStateChange) === null || _b === void 0 ? void 0 : _b.call(_a, true);
                    this.startPingInterval();
                }
            }
            catch (error) {
                console.error('Failed to connect WebSocket via IPC:', error);
                (_d = (_c = this.callbacks).onError) === null || _d === void 0 ? void 0 : _d.call(_c, 'WebSocket connection failed');
                this.connected = false;
            }
        });
    }
    connect() {
        if (this.connected) {
            console.log('WebSocket already connected');
            return;
        }
        this.isManuallyDisconnected = false;
        console.log(`Connecting via IPC WebSocket proxy: ${this.url}`);
        // Use IPC-based WebSocket instead of direct connection
        this.connectViaIPC();
    }
    connectDirectly() {
        var _a, _b;
        // Fallback to direct connection (kept for reference but not used)
        try {
            this.ws = new WebSocket(this.url);
            console.log('WebSocket created, state:', this.ws.readyState);
            this.ws.onopen = () => {
                var _a, _b;
                console.log('WebSocket connected successfully');
                this.reconnectAttempts = 0;
                (_b = (_a = this.callbacks).onConnectionStateChange) === null || _b === void 0 ? void 0 : _b.call(_a, true);
                // Start ping interval to keep connection alive
                this.startPingInterval();
            };
            this.ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    this.handleMessage(message);
                }
                catch (error) {
                    console.error('Failed to parse WebSocket message:', error);
                }
            };
            this.ws.onerror = (error) => {
                var _a, _b;
                console.error('WebSocket error:', error);
                (_b = (_a = this.callbacks).onError) === null || _b === void 0 ? void 0 : _b.call(_a, 'WebSocket connection error');
            };
            this.ws.onclose = (event) => {
                var _a, _b;
                console.log('WebSocket disconnected:', event.code, event.reason);
                (_b = (_a = this.callbacks).onConnectionStateChange) === null || _b === void 0 ? void 0 : _b.call(_a, false);
                this.stopPingInterval();
                // Attempt reconnection if not manually disconnected
                if (!this.isManuallyDisconnected && this.reconnectAttempts < this.maxReconnectAttempts) {
                    this.scheduleReconnect();
                }
            };
        }
        catch (error) {
            console.error('Failed to create WebSocket:', error);
            (_b = (_a = this.callbacks).onError) === null || _b === void 0 ? void 0 : _b.call(_a, `Failed to connect: ${error}`);
        }
    }
    handleMessage(message) {
        var _a, _b, _c, _d, _e, _f, _g, _h, _j, _k, _l, _m, _o, _p, _q, _r;
        switch (message.type) {
            case 'profile_loaded':
                (_b = (_a = this.callbacks).onProfileLoaded) === null || _b === void 0 ? void 0 : _b.call(_a, message.name, message.models);
                break;
            case 'stage_started':
                (_d = (_c = this.callbacks).onStageStarted) === null || _d === void 0 ? void 0 : _d.call(_c, message.stage, message.model);
                break;
            case 'stream_chunk':
                (_f = (_e = this.callbacks).onStreamChunk) === null || _f === void 0 ? void 0 : _f.call(_e, message.stage, message.chunk);
                break;
            case 'stage_progress':
                (_h = (_g = this.callbacks).onStageProgress) === null || _h === void 0 ? void 0 : _h.call(_g, message.stage, message.percentage, message.tokens);
                break;
            case 'stage_completed':
                (_k = (_j = this.callbacks).onStageCompleted) === null || _k === void 0 ? void 0 : _k.call(_j, message.stage, message.tokens, message.cost);
                break;
            case 'consensus_complete':
                (_m = (_l = this.callbacks).onConsensusComplete) === null || _m === void 0 ? void 0 : _m.call(_l, message.result, message.total_tokens, message.total_cost);
                break;
            case 'error':
                (_p = (_o = this.callbacks).onError) === null || _p === void 0 ? void 0 : _p.call(_o, message.message);
                break;
            case 'ai_helper_decision':
                (_r = (_q = this.callbacks).onAIHelperDecision) === null || _r === void 0 ? void 0 : _r.call(_q, message.direct_mode, message.reason);
                break;
            default:
                console.warn('Unknown message type:', message.type);
        }
    }
    scheduleReconnect() {
        const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
        this.reconnectAttempts++;
        console.log(`Scheduling reconnect attempt ${this.reconnectAttempts} in ${delay}ms`);
        this.reconnectTimer = window.setTimeout(() => {
            this.connect();
        }, delay);
    }
    startPingInterval() {
        // Send ping every 30 seconds to keep connection alive
        this.pingInterval = window.setInterval(() => {
            var _a;
            if (((_a = this.ws) === null || _a === void 0 ? void 0 : _a.readyState) === WebSocket.OPEN) {
                this.ws.send(JSON.stringify({ type: 'ping' }));
            }
        }, 30000);
    }
    stopPingInterval() {
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
            this.pingInterval = null;
        }
    }
    startConsensus(query, profile, conversationId, context) {
        var _a, _b, _c, _d;
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.connected) {
                (_b = (_a = this.callbacks).onError) === null || _b === void 0 ? void 0 : _b.call(_a, 'WebSocket not connected');
                return;
            }
            const message = {
                type: 'start_consensus',
                query,
                profile,
                conversation_id: conversationId,
                context: context
            };
            try {
                yield window.websocketAPI.send(JSON.stringify(message));
                console.log('Sent consensus request via IPC with conversation_id:', conversationId);
            }
            catch (error) {
                console.error('Failed to send consensus request:', error);
                (_d = (_c = this.callbacks).onError) === null || _d === void 0 ? void 0 : _d.call(_c, 'Failed to send message');
            }
        });
    }
    cancelConsensus() {
        var _a;
        if (((_a = this.ws) === null || _a === void 0 ? void 0 : _a.readyState) === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({ type: 'cancel_consensus' }));
        }
    }
    disconnect() {
        this.isManuallyDisconnected = true;
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        this.stopPingInterval();
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
    }
    isConnected() {
        return this.connected;
    }
}
exports.ConsensusWebSocket = ConsensusWebSocket;
// Stage display mapping
exports.STAGE_DISPLAY_NAMES = {
    'Generator': 'generator',
    'Refiner': 'refiner',
    'Validator': 'validator',
    'Curator': 'curator'
};
// Utility to format tokens and cost
function formatTokens(tokens) {
    if (tokens >= 1000000) {
        return `${(tokens / 1000000).toFixed(2)}M`;
    }
    else if (tokens >= 1000) {
        return `${(tokens / 1000).toFixed(1)}k`;
    }
    return tokens.toString();
}
exports.formatTokens = formatTokens;
function formatCost(cost) {
    if (cost < 0.01) {
        return `$${(cost * 100).toFixed(3)}¢`;
    }
    return `$${cost.toFixed(3)}`;
}
exports.formatCost = formatCost;
//# sourceMappingURL=consensus-websocket.js.map