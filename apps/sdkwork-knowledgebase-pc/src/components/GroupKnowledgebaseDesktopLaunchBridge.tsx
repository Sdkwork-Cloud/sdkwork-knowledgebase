import { useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  createGroupKnowledgebaseDesktopLaunchHost,
  parseGroupKnowledgebaseLaunchRoute,
} from 'sdkwork-knowledgebase-pc-core/host';

const desktopLaunchHost = createGroupKnowledgebaseDesktopLaunchHost();

/** Routes validated native launches into the one reusable standalone window. */
export function GroupKnowledgebaseDesktopLaunchBridge() {
  const navigate = useNavigate();
  const lastRouteRef = useRef<string | null>(null);

  useEffect(() => {
    const openRoute = (candidate: string) => {
      const route = parseGroupKnowledgebaseLaunchRoute(candidate);
      if (!route || lastRouteRef.current === route) {
        return;
      }
      lastRouteRef.current = route;
      navigate(route);
    };

    const unsubscribe = desktopLaunchHost.subscribe(openRoute);
    void desktopLaunchHost.takePending().then((route) => {
      if (route) {
        openRoute(route);
      }
    });
    return unsubscribe;
  }, [navigate]);

  return null;
}
