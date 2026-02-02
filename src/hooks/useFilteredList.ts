import { useMemo } from "react";

/**
 * Hook for filtering and searching lists with memoization.
 *
 * @template T - The type of items in the list
 * @param items - The array of items to filter
 * @param searchQuery - The search query string
 * @param searchFields - Array of field names to search within
 * @param filterFn - Optional custom filter function for additional filtering
 * @returns Filtered array of items
 *
 * @example
 * ```tsx
 * interface Package {
 *   name: string;
 *   version: string;
 *   is_outdated: boolean;
 * }
 *
 * const filteredPackages = useFilteredList(
 *   packages,
 *   searchQuery,
 *   ["name", "version"],
 *   (pkg) => showOutdatedOnly ? pkg.is_outdated : true
 * );
 * ```
 *
 * @example
 * ```tsx
 * // Simple search without additional filtering
 * const filteredItems = useFilteredList(
 *   items,
 *   searchQuery,
 *   ["name", "description"]
 * );
 * ```
 *
 * @example
 * ```tsx
 * // With type filter
 * const filteredBrewPackages = useFilteredList(
 *   packages,
 *   searchQuery,
 *   ["name"],
 *   (pkg) => {
 *     if (filter === "formulae") return !pkg.is_cask;
 *     if (filter === "casks") return pkg.is_cask;
 *     if (filter === "outdated") return pkg.is_outdated;
 *     return true;
 *   }
 * );
 * ```
 */
export function useFilteredList<T>(
  items: T[],
  searchQuery: string,
  searchFields: (keyof T)[],
  filterFn?: (item: T) => boolean
): T[] {
  return useMemo(() => {
    let result = items;

    // Apply custom filter function first
    if (filterFn) {
      result = result.filter(filterFn);
    }

    // Apply search query if present
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      result = result.filter((item) =>
        searchFields.some((field) => {
          const value = item[field];
          return typeof value === "string" && value.toLowerCase().includes(query);
        })
      );
    }

    return result;
  }, [items, searchQuery, searchFields, filterFn]);
}

export default useFilteredList;
