interface User {
  name: string;
  age: number;
}

type UserRole = "admin" | "user" | "guest";

function createUser(name: string, age: number): User {
  return { name, age };
}