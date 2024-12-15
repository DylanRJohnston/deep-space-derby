let pendingEvents = [];
function sendGameEvent(event) {
    if (typeof globalThis['innerSendGameEvent'] !== 'function') {
        console.warn('pushing message before module initialised');
        console.warn(event)
        pendingEvents.push(event);
    } else {
        globalThis['innerSendGameEvent'](event);
    }
}