package app.grapegram;

import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.text.InputType;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.Toast;

import org.telegram.ui.ActionBar.AlertDialog;
import org.telegram.ui.ActionBar.BaseFragment;
import org.telegram.ui.LaunchActivity;

import tw.nekomimi.nekogram.helpers.AppRestartHelper;

/** Required on first launch: user supplies their own api_id / api_hash from my.telegram.org. */
public final class GrapeApiGate {

    private static final String PREFS = "nekox_config";
    private static boolean shown;

    private GrapeApiGate() {
    }

    public static boolean hasKeys() {
        Context c = org.telegram.messenger.ApplicationLoader.applicationContext;
        if (c == null) {
            return false;
        }
        SharedPreferences p = c.getSharedPreferences(PREFS, Context.MODE_PRIVATE);
        String hash = p.getString("custom_app_hash", "");
        return p.getInt("custom_api", 0) == 3
                && p.getInt("custom_app_id", 0) != 0
                && hash != null && hash.length() == 32;
    }

    public static void maybeAsk(BaseFragment fragment) {
        if (hasKeys() || shown || fragment == null) {
            return;
        }
        Context context = fragment.getParentActivity();
        if (context == null) {
            return;
        }
        shown = true;

        EditText idInput = new EditText(context);
        idInput.setInputType(InputType.TYPE_CLASS_NUMBER);
        idInput.setHint("api_id (my.telegram.org)");
        EditText hashInput = new EditText(context);
        hashInput.setInputType(InputType.TYPE_CLASS_TEXT);
        hashInput.setHint("api_hash (32 символа)");

        LinearLayout box = new LinearLayout(context);
        box.setOrientation(LinearLayout.VERTICAL);
        int pad = org.telegram.messenger.AndroidUtilities.dp(20);
        box.setPadding(pad, pad / 2, pad, pad);
        box.addView(idInput);
        box.addView(hashInput);

        new AlertDialog.Builder(context)
                .setTitle("grapeGram — API ключи")
                .setMessage("Введите свои api_id и api_hash с my.telegram.org → API Development Tools. "
                        + "Ключи сохраняются только на этом устройстве. Без них вход невозможен.")
                .setView(box)
                .setCancelable(false)
                .setPositiveButton("Сохранить и перезапустить", (dialog, which) -> {
                    String idText = idInput.getText().toString().trim();
                    String hash = hashInput.getText().toString().trim();
                    int id;
                    try {
                        id = Integer.parseInt(idText);
                    } catch (NumberFormatException e) {
                        id = 0;
                    }
                    if (id == 0 || hash.length() != 32) {
                        Toast.makeText(context, "Нужны api_id (число) и api_hash (32 символа)",
                                Toast.LENGTH_LONG).show();
                        shown = false;
                        maybeAsk(fragment);
                        return;
                    }
                    SharedPreferences.Editor editor = context
                            .getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit();
                    editor.putInt("custom_api", 3);
                    editor.putInt("custom_app_id", id);
                    editor.putString("custom_app_hash", hash);
                    editor.commit();
                    AppRestartHelper.triggerRebirth(context, new Intent(context, LaunchActivity.class));
                })
                .show();
    }
}
