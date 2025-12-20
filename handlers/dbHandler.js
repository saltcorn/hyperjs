// /db/{action}
async (req, res) => {
    const { action } = req.params;

    try {
        if (action === "list") {
            // List all users from the database
            Deno.core.ops.op_log("Fetching all users from database...");
            const result = await Deno.core.ops.op_db_query(
                "SELECT * FROM users ORDER BY id",
                []
            );
            const users = JSON.parse(result);
            res.json({
                action: "list",
                count: users.length,
                users: users,
            });
        } else if (action === "create") {
            // Create a new user
            const { name } = req.query;
            if (!name) {
                res.status(400).json({ error: "Name parameter required" });
                return;
            }

            Deno.core.ops.op_log(`Creating user: ${name}`);
            const result = await Deno.core.ops.op_db_query(
                "INSERT INTO users (name, email) VALUES (?, ?)",
                [name, `${name.toLowerCase()}@example.com`]
            );
            const insertResult = JSON.parse(result);
            res.json({
                action: "create",
                name: name,
                result: insertResult,
            });
        } else if (action === "count") {
            // Count total users
            Deno.core.ops.op_log("Counting users...");
            const result = await Deno.core.ops.op_db_query(
                "SELECT COUNT(*) as total FROM users",
                []
            );
            const countResult = JSON.parse(result);
            res.json({
                action: "count",
                total: countResult[0].total,
            });
        } else {
            res.status(404).json({
                error: "Unknown action",
                validActions: ["list", "create", "count"],
            });
        }
    } catch (err) {
        Deno.core.ops.op_log(`Database error: ${err.message}`);
        res.status(500).json({
            error: "Database operation failed",
            message: err.message,
        });
    }
};
