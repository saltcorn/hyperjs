/// PUT /users/{userId}
async (req, res) => {
    const { userId } = req.params;
    const { name, email } = req.query;

    if (!name && !email) {
        res.status(400).json({
            error: "At least one field (name or email) must be provided",
        });
        return;
    }

    Deno.core.ops.op_log(`Updating user ${userId}`);

    // Build dynamic update query
    const updates = [];
    const values = [];

    if (name) {
        updates.push("name = ?");
        values.push(name);
    }
    if (email) {
        updates.push("email = ?");
        values.push(email);
    }

    values.push(userId);

    const query = `UPDATE users SET ${updates.join(", ")} WHERE id = ?`;

    const result = await Deno.core.ops.op_db_query(query, values);
    const updateResult = JSON.parse(result);

    res.json({
        message: "User updated successfully",
        userId,
        updated: { name, email },
        result: updateResult,
    });
};
