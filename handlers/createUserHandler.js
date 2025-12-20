// POST /users
async (req, res) => {
    const { name, email } = req.query;

    if (!name || !email) {
        res.status(400).json({
            error: "Missing required fields",
            required: ["name", "email"],
        });
        return;
    }

    Deno.core.ops.op_log(`Creating user: ${name} (${email})`);

    const result = await Deno.core.ops.op_db_query(
        "INSERT INTO users (name, email) VALUES (?, ?)",
        [name, email]
    );

    const insertResult = JSON.parse(result);

    res.status(201).json({
        message: "User created successfully",
        user: {
            name,
            email,
        },
        result: insertResult,
    });
};
