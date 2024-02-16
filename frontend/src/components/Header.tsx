import cx from 'clsx';
import { useState } from 'react';
import {
  Container,
  Avatar,
  UnstyledButton,
  Group,
  Text,
  Menu,
  Tabs,
  Burger,
  rem,
  Button,
  ActionIcon,
  useMantineColorScheme,
  useComputedColorScheme,
  AppShellHeader,
} from '@mantine/core';
import { useDisclosure } from '@mantine/hooks';
import {
  IconLogout,
  IconSettings,
  IconChevronDown,
  IconPlus,
  IconBook2,
  IconTriangleInvertedFilled,
  IconSun,
  IconMoon,
} from '@tabler/icons-react';
import logo from '../assets/logos.png';
import classes from './Header.module.css';
import { Link, useNavigate } from 'react-router-dom';
import { useUser } from '../UserContext';

const tabs = ['Home', 'JSON Playground'];

export default function Header() {
  const [opened, { toggle }] = useDisclosure(false);
  const [userMenuOpened, setUserMenuOpened] = useState(false);
  const navigate = useNavigate();

  const { user, logout } = useUser();

  const { setColorScheme } = useMantineColorScheme();
  const computedColorScheme = useComputedColorScheme('light', {
    getInitialValueInEffect: true,
  });

  const items = tabs.map((tab) => (
    <Tabs.Tab
      value={tab}
      key={tab}
      onClick={() => {
        switch (tab) {
          case 'JSON Playground':
            navigate('/json-ir');
            break;
          default:
            navigate('/');
        }
      }}
    >
      {tab}
    </Tabs.Tab>
  ));

  return (
    <AppShellHeader>
      <Container className={classes.mainSection} size="md">
        <Group justify="space-between">
          <img src={logo} alt="Logo" style={{ width: 40 }} />

          <Burger opened={opened} onClick={toggle} hiddenFrom="xs" size="sm" />

          <Group justify="flex-end">
            {user && (
              <div>
                <Menu>
                  <Menu.Target>
                    <Button
                      variant="default"
                      size="xs"
                      radius="md"
                      style={{ padding: '0 0.6rem' }}
                    >
                      <IconPlus
                        size="1.3rem"
                        style={{ marginRight: '0.3rem' }}
                      />
                      <IconTriangleInvertedFilled size="0.5rem" />
                    </Button>
                  </Menu.Target>

                  <Menu.Dropdown>
                    <Link
                      to="/new-circuit"
                      style={{
                        textDecoration: 'none',
                      }}
                    >
                      <Menu.Item
                        leftSection={
                          <IconBook2
                            style={{ width: rem(14), height: rem(14) }}
                          />
                        }
                      >
                        New repository
                      </Menu.Item>
                    </Link>
                  </Menu.Dropdown>
                </Menu>
              </div>
            )}

            <ActionIcon
              onClick={() =>
                setColorScheme(
                  computedColorScheme === 'light' ? 'dark' : 'light',
                )
              }
              variant="default"
              size="md"
              aria-label="Toggle color scheme"
            >
              {computedColorScheme === 'dark' ? (
                <IconSun stroke={1.5} size={'1.2rem'} />
              ) : (
                <IconMoon stroke={1.5} size={'1.2rem'} />
              )}
            </ActionIcon>

            {user ? (
              <Menu
                width={260}
                position="bottom-end"
                transitionProps={{ transition: 'pop-top-right' }}
                onClose={() => setUserMenuOpened(false)}
                onOpen={() => setUserMenuOpened(true)}
                withinPortal
              >
                <Menu.Target>
                  <UnstyledButton
                    className={cx(classes.user, {
                      [classes.userActive]: userMenuOpened,
                    })}
                  >
                    <Group gap={7}>
                      <Avatar
                        src={user.image}
                        alt={user.name}
                        radius="xl"
                        size={20}
                      />
                      <Text fw={500} size="sm" lh={1} mr={3}>
                        {user.name}
                      </Text>
                      <IconChevronDown
                        style={{ width: rem(12), height: rem(12) }}
                        stroke={1.5}
                      />
                    </Group>
                  </UnstyledButton>
                </Menu.Target>
                <Menu.Dropdown>
                  <Menu.Item
                    leftSection={
                      <IconSettings
                        style={{ width: rem(16), height: rem(16) }}
                        stroke={1.5}
                      />
                    }
                  >
                    Account settings
                  </Menu.Item>
                  <Menu.Item
                    leftSection={
                      <IconLogout
                        style={{ width: rem(16), height: rem(16) }}
                        stroke={1.5}
                      />
                    }
                    onClick={() => logout()}
                  >
                    Logout
                  </Menu.Item>
                </Menu.Dropdown>
              </Menu>
            ) : (
              <a href="/auth/google">
                <Button variant="outline" size="xs">
                  Log In
                </Button>
              </a>
            )}
          </Group>
        </Group>
      </Container>
      <Container size="md">
        <Tabs
          defaultValue="Home"
          variant="outline"
          visibleFrom="sm"
          classNames={{
            root: classes.tabs,
            list: classes.tabsList,
            tab: classes.tab,
          }}
        >
          <Tabs.List>{items}</Tabs.List>
        </Tabs>
      </Container>
    </AppShellHeader>
  );
}
