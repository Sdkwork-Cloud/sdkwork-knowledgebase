import { useState, useEffect, useCallback } from 'react';

export interface UseLocalStorageOptions {
  enabled?: boolean;
}

export function useLocalStorage<T>(
  key: string,
  initialValue: T,
  options: UseLocalStorageOptions = {},
): [T, (value: T | ((val: T) => T)) => void] {
  const enabled = options.enabled ?? true;
  const [storedValue, setStoredValue] = useState<T>(() => {
    if (!enabled || typeof window === "undefined") {
      return initialValue;
    }
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initialValue;
    } catch (error) {
      console.warn(`Error reading localStorage key "${key}":`, error);
      return initialValue;
    }
  });

  const setValue = useCallback((value: T | ((val: T) => T)) => {
    try {
      setStoredValue((prev) => {
        const valueToStore = value instanceof Function ? value(prev) : value;
        if (enabled && typeof window !== "undefined") {
          window.localStorage.setItem(key, JSON.stringify(valueToStore));
          queueMicrotask(() => {
            window.dispatchEvent(new CustomEvent('local-storage', { detail: { key, value: valueToStore } }));
          });
        }
        return valueToStore;
      });
    } catch (error) {
      console.warn(`Error setting localStorage key "${key}":`, error);
    }
  }, [enabled, key]);

  useEffect(() => {
    if (!enabled) {
      return;
    }
    const handleStorageChange = (e: Event) => {
      const customEvent = e as CustomEvent;
      if (customEvent.detail.key === key) {
        setStoredValue(customEvent.detail.value);
      }
    };
    
    const handleNativeStorage = (e: StorageEvent) => {
       if (e.key === key && e.newValue) {
           try {
             setStoredValue(JSON.parse(e.newValue));
           } catch { }
       }
    };

    window.addEventListener('local-storage', handleStorageChange);
    window.addEventListener('storage', handleNativeStorage);
    return () => {
      window.removeEventListener('local-storage', handleStorageChange);
      window.removeEventListener('storage', handleNativeStorage);
    };
  }, [enabled, key]);

  return [storedValue, setValue];
}
