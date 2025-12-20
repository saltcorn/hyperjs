/* DELETE /users/{userId} */
async (req, res) => {
    const { userId } = req.params;

    Deno.core.ops.op_log(`Deleting user ${userId}`);

    // First check if user exists
    const checkResult = await Deno.core.ops.op_db_query(
        "SELECT * FROM users WHERE id = ?",
        [userId]
    );
    const users = JSON.parse(checkResult);

    if (users.length === 0) {
        res.status(404).json({
            error: "User not found",
            userId,
        });
        return;
    }

    // Delete the user
    const result = await Deno.core.ops.op_db_query(
        "DELETE FROM users WHERE id = ?",
        [userId]
    );
    const deleteResult = JSON.parse(result);

    res.json({
        message: "User deleted successfully",
        userId,
        deletedUser: users[0],
        result: deleteResult,
    });
};
