import { jwtDecode } from 'jwt-decode';

export type Claims = {
  sub: string;
  exp: number;
  name?: string;
  picture?: string;
};

export function getUserClaims(token: string) {
  return jwtDecode<Claims>(token);
}
