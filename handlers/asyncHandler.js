// /async/{delay}
async (req, res) => {
    const { delay } = req.params;
    const delayMs = parseInt(delay) || 1000;

    Deno.core.ops.op_log(`Starting async operation with ${delayMs}ms delay...`);

    // Use the async op_sleep to simulate an async operation
    const startTime = Date.now();
    await Deno.core.ops.op_sleep(delayMs);
    const endTime = Date.now();

    Deno.core.ops.op_log(`Async operation completed!`);

    res.json({
        message: "Async operation completed",
        requestedDelay: delayMs,
        actualDelay: endTime - startTime,
        timestamp: Date.now(),
    });
};
