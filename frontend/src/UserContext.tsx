import {
  createContext,
  useState,
  useContext,
  ReactNode,
  useEffect,
} from 'react';
import { getUserClaims } from './jwt';
import { useQueryClient } from 'react-query';

interface IUser {
  sub: string;
  name: string;
  image: string;
  auth_token: string;
}

interface IUserContext {
  user?: IUser;
  setUserData: (newUser: IUser) => void;
  logout: () => void;
}

const UserContext = createContext<IUserContext | null>(null);

export const UserProvider = ({ children }: { children: ReactNode }) => {
  const [userData, setUserData] = useState<IUser | undefined>(() => {
    const token = localStorage.getItem('token');

    if (token) {
      const claims = getUserClaims(token);

      return {
        sub: claims.sub,
        name: claims.name || 'Unknown',
        image: claims.picture || '',
        auth_token: token,
      };
    }
  });

  const queryClient = useQueryClient();

  useEffect(() => {
    if (userData) {
      localStorage.setItem('token', userData.auth_token);
    } else {
      localStorage.removeItem('token');
    }
  }, [userData]);

  return (
    <UserContext.Provider
      value={{
        user: userData,
        setUserData: (newUser: IUser) => {
          setUserData(newUser);
        },
        logout: () => {
          setUserData(undefined);
          queryClient.clear();
          window.location.href = '/';
        },
      }}
    >
      {children}
    </UserContext.Provider>
  );
};

export const useUser = () => {
  const context = useContext(UserContext);

  if (!context) {
    throw new Error('useUser must be used within a UserProvider');
  }

  return context;
};
