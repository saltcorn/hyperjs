# Handlers Summary

This document lists all the JavaScript handlers currently loaded by the macro system.

## Handler Files

### GET Handlers

1. **helloHandler.js**

    - Route: `GET /hello/{name}`
    - Description: Simple hello world handler with name parameter
    - Comment style: `//`

2. **dataHandler.js**

    - Route: `GET /data/{id}`
    - Description: Returns JSON data with ID and timestamp
    - Comment style: `//`

3. **userHandler.js**

    - Route: `GET /users/{userId}`
    - Description: Gets user information with optional include parameter
    - Comment style: `//`

4. **asyncHandler.js**

    - Route: `GET /async/{delay}`
    - Description: Demonstrates async operations with configurable delay
    - Comment style: `//`

5. **dbHandler.js**
    - Route: `GET /db/{action}`
    - Description: Database operations (list, create, count)
    - Comment style: `//`

### POST Handlers

6. **createUserHandler.js**
    - Route: `POST /users`
    - Description: Creates a new user in the database
    - Comment style: `//`
    - Query params: `name`, `email`

### PUT Handlers

7. **updateUserHandler.js**
    - Route: `PUT /users/{userId}`
    - Description: Updates an existing user
    - Comment style: `///`
    - Query params: `name`, `email` (at least one required)

### DELETE Handlers

8. **deleteUserHandler.js**
    - Route: `DELETE /users/{userId}`
    - Description: Deletes a user from the database
    - Comment style: `/* */`

## Testing the Handlers

### Using curl

```bash
# GET handlers
curl http://localhost:8080/hello/World
curl http://localhost:8080/data/123
curl http://localhost:8080/users/1?include=profile
curl http://localhost:8080/async/1000
curl http://localhost:8080/db/list

# POST handler
curl -X POST "http://localhost:8080/users?name=David&email=david@example.com"

# PUT handler
curl -X PUT "http://localhost:8080/users/1?name=UpdatedName&email=updated@example.com"

# DELETE handler
curl -X DELETE http://localhost:8080/users/1
```

## Adding New Handlers

To add a new handler:

1. Create a new `.js` file in the `handlers/` directory
2. Add the method and route in the first line comment
3. Implement the handler function
4. Run `cargo build`
5. Start the server with `cargo run`

Example:

```javascript
// POST /products
async (req, res) => {
    const { name, price } = req.query;
    res.status(201).json({ name, price });
};
```

The handler will be automatically loaded and the route registered!
