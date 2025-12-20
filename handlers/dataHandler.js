// /data/{id}
async (req, res) => {
    const { id } = req.params;

    res.json({
        id,
        message: "Data fetched",
        timestamp: Date.now(),
    });
};
