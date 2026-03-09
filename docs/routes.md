### register a user, return confirmation ID
POST /users

### confirm a user registration with code sent via email
POST /confirmations/registration/{confirmation_id}
{
    // example data
    "code": "A6C9"
}

### reject a user registration with code sent via email
DELETE /confirmations/registration/{confirmation_id}
{
    // example data
    "code": "A6C9"
}