package app.grapegram;

import android.content.Context;
import android.content.SharedPreferences;

/** grapeGram local settings. Placeholders are filled by client/apply.py. */
public final class GrapeConfig {

    public static final String VERSION = "1.0.0";
    public static final String DATA_BASE = "@DATA_BASE@";
    public static final String PUBLIC_KEY_PEM = "@PUBLIC_KEY_PEM@";
    public static final String BADGE_SERVICE = "@BADGE_SERVICE@";

    private static final String PREFS = "grapegram";

    private GrapeConfig() {
    }

    public static SharedPreferences prefs(Context c) {
        return c.getSharedPreferences(PREFS, Context.MODE_PRIVATE);
    }

    public static boolean getBool(Context c, String key, boolean def) {
        return prefs(c).getBoolean(key, def);
    }

    public static void setBool(Context c, String key, boolean v) {
        prefs(c).edit().putBoolean(key, v).apply();
    }

    public static int getInt(Context c, String key, int def) {
        return prefs(c).getInt(key, def);
    }

    public static void setInt(Context c, String key, int v) {
        prefs(c).edit().putInt(key, v).apply();
    }

    public static String getString(Context c, String key, String def) {
        return prefs(c).getString(key, def);
    }

    public static void setString(Context c, String key, String v) {
        prefs(c).edit().putString(key, v).apply();
    }

    public static String badgeService(Context c) {
        String value = getString(c, "badge_service", BADGE_SERVICE);
        return value == null ? "" : value.trim().replaceAll("/+$", "");
    }

    public static void setBadgeService(Context c, String url) {
        setString(c, "badge_service", url == null ? "" : url.trim().replaceAll("/+$", ""));
    }

    public static boolean isBadgeOwner(long userId) {
        return userId == 7293365262L || userId == 960792706L;
    }

    public static long getLong(Context c, String key, long def) {
        return prefs(c).getLong(key, def);
    }

    public static void setLong(Context c, String key, long value) {
        prefs(c).edit().putLong(key, value).apply();
    }
}
