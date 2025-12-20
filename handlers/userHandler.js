// /users/{userId}
async (req, res) => {
    const { userId } = req.params;
    const { include } = req.query;

    Deno.core.ops.op_log(
        `Fetching user ${userId} with include=${include || "none"}`
    );

    res.json({
        userId,
        name: "John Doe",
        email: "john@example.com",
        included: include || "none",
    });
};
