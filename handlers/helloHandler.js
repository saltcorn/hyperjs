// /hello/{name}
async (req, res) => {
    const { name } = req.params;

    // Note: setTimeout is not available in this runtime
    // Use a simple async operation instead
    await Promise.resolve();

    res.send(`Hello ${name}!`);
};
