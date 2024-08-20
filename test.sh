# Create a new user
echo "Creating a new user..."
CREATE_RESPONSE=$(curl -s -X POST http://localhost:8080/users \
     -H "Content-Type: application/json" \
     -d '{"username": "johndoe", "email": "john@example.com"}')
echo $CREATE_RESPONSE
USER_ID=$(echo $CREATE_RESPONSE | sed 's/.*"id":\([0-9]*\).*/\1/')

# Get the user
echo "\nGetting the user..."
curl -s http://localhost:8080/users/$USER_ID

# Update the user
echo "\nUpdating the user..."
curl -s -X PUT http://localhost:8080/users/$USER_ID \
     -H "Content-Type: application/json" \
     -d '{"username": "johndoe_updated", "email": "john_new@example.com"}'

# Get the user again to verify update
echo "\nGetting the updated user..."
curl -s http://localhost:8080/users/$USER_ID

# Delete the user
echo "\nDeleting the user..."
curl -s -X DELETE http://localhost:8080/users/$USER_ID

# Try to get the deleted user (should return an error)
echo "\nTrying to get the deleted user..."
curl -s http://localhost:8080/users/$USER_ID