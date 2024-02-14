import { createContext, useState, useContext, ReactNode } from 'react';

interface IUser {
  name: string;
  image: string;
  token: string;
}

interface IUserContext {
  user?: IUser;
  setUserData: (newUser: IUser) => void;
  logout: () => void;
}

const UserContext = createContext<IUserContext | null>(null);

export const UserProvider = ({ children }: { children: ReactNode }) => {
  const [userData, setUserData] = useState<IUser | undefined>(undefined);

  return (
    <UserContext.Provider
      value={{
        user: userData,
        setUserData: (newUser: IUser) => {
          setUserData(newUser);
        },
        logout: () => {
          setUserData(undefined);
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
