import axios from 'axios';
import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { useUser } from '../UserContext';
import { getUserClaims } from '../jwt';

const Auth = () => {
  const location = useLocation();
  const navigate = useNavigate();

  const query = new URLSearchParams(location.search);
  const code = query.get('code');
  const { setUserData } = useUser();

  useEffect(() => {
    if (code) {
      axios
        .post<{
          id_token: string;
          access_token: string;
          refresh_token: string;
          expires_in: number;
          token_type: string;
        }>(
          'https://zkcir.auth.us-east-1.amazoncognito.com/oauth2/token',
          {
            client_id: '4urlcgg95ohqfelj39qnfoe0ig',
            redirect_uri: window.location.origin + '/auth/callback',
            grant_type: 'authorization_code',
            code: code,
          },
          {
            headers: {
              'Content-Type': 'application/x-www-form-urlencoded',
            },
          },
        )
        .then((response) => {
          const claims = getUserClaims(response.data.id_token);

          setUserData({
            sub: claims.sub,
            name: claims.name || 'Unknown',
            image: claims.picture || '',
            auth_token: response.data.id_token,
          });

          navigate('/');
        })
        .catch((error) => console.error(error));
    }
  }, [code, navigate, setUserData]);

  return null;
};

export default Auth;
