package app.grapegram;

import android.content.ComponentName;
import android.content.Context;
import android.content.pm.PackageManager;

import org.telegram.messenger.R;

/** Resolves the icon alias currently enabled by Android's package manager. */
public final class GrapeIconResolver {
    private static final String[] ALIASES = {
            "GrapeIconOriginal", "GrapeIconAmethyst", "GrapeIconMosaic", "GrapeIconOnyx",
            "GrapeIconSilk", "GrapeIconNeon", "GrapeIconOrbit", "GrapeIconGraphite", "GrapeIconViolet",
            "GrapeCherryIconCherry",
            "GrapeCherryIconDark_cherry",
            "GrapeCherryIconDark_cherry_bra",
            "GrapeCherryIconWhite_cherry",
            "GrapeCherryIconWhite_cherry_bra",
            "GrapeCherryIconLaguna_cherry",
            "GrapeCherryIconAqua_cherry",
            "GrapeCherryIconGreen_cherry",
            "GrapeCherryIconLavanda_cherry",
            "GrapeCherryIconViolet_sunset_cherry",
            "GrapeCherryIconViolet_sunset_cherry_bra",
            "GrapeCherryIconSunset_cherry",
            "GrapeCherryIconSunrise_cherry",
            "GrapeCherryIconPalms_cherry",
            "GrapeCherryIconPalms_cherry_bra",
            "GrapeCherryIconTurbo_cherry",
            "GrapeCherryIconNox_cherry",
            "GrapeCherryIconDark_ny"
    };
    private static final int[] ICONS = {
            R.drawable.grape_icon_original,
            R.drawable.grape_icon_amethyst,
            R.drawable.grape_icon_mosaic,
            R.drawable.grape_icon_onyx,
            R.drawable.grape_icon_silk,
            R.drawable.grape_icon_neon,
            R.drawable.grape_icon_orbit,
            R.drawable.grape_icon_graphite,
            R.drawable.grape_icon_violet,
            R.mipmap.grape_cherry_icon_cherry,
            R.mipmap.grape_cherry_icon_dark_cherry,
            R.mipmap.grape_cherry_icon_dark_cherry_bra,
            R.mipmap.grape_cherry_icon_white_cherry,
            R.mipmap.grape_cherry_icon_white_cherry_bra,
            R.mipmap.grape_cherry_icon_laguna_cherry,
            R.mipmap.grape_cherry_icon_aqua_cherry,
            R.mipmap.grape_cherry_icon_green_cherry,
            R.mipmap.grape_cherry_icon_lavanda_cherry,
            R.mipmap.grape_cherry_icon_violet_sunset_cherry,
            R.mipmap.grape_cherry_icon_violet_sunset_cherry_bra,
            R.mipmap.grape_cherry_icon_sunset_cherry,
            R.mipmap.grape_cherry_icon_sunrise_cherry,
            R.mipmap.grape_cherry_icon_palms_cherry,
            R.mipmap.grape_cherry_icon_palms_cherry_bra,
            R.mipmap.grape_cherry_icon_turbo_cherry,
            R.mipmap.grape_cherry_icon_nox_cherry,
            R.mipmap.grape_cherry_icon_dark_ny
    };

    private GrapeIconResolver() {
    }

    public static int getSelectedIcon(Context context) {
        if (context == null) {
            return R.drawable.grape_icon_original;
        }
        PackageManager pm = context.getPackageManager();
        for (int i = 0; i < ALIASES.length; i++) {
            try {
                ComponentName component = new ComponentName(
                        context, "org.telegram.messenger." + ALIASES[i]);
                if (pm.getComponentEnabledSetting(component)
                        == PackageManager.COMPONENT_ENABLED_STATE_ENABLED) {
                    return ICONS[i];
                }
            } catch (Throwable ignored) {
                // A missing alias must not break the settings screen.
            }
        }
        return R.drawable.grape_icon_original;
    }
}
