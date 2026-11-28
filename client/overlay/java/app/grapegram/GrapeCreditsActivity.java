package app.grapegram;

import android.content.Context;
import android.view.View;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import org.telegram.messenger.R;
import org.telegram.messenger.browser.Browser;
import org.telegram.ui.ActionBar.Theme;
import org.telegram.ui.Cells.TextInfoPrivacyCell;
import org.telegram.ui.Cells.TextSettingsCell;

import tw.nekomimi.nekogram.settings.BaseNekoSettingsActivity;
import tw.nekomimi.nekogram.ui.cells.HeaderCell;

/** Full credits page: upstream projects, communities, design and license. */
public class GrapeCreditsActivity extends BaseNekoSettingsActivity {

    private static final String CHERRYGRAM = "https://github.com/arsLan4k1390/Cherrygram";
    private static final String EXTERAGRAM = "https://github.com/exteraSquad/exteraGram";
    private static final String EXTERALESS = "https://github.com/exteraless/exteraless";

    private int projectsHeaderRow;
    private int cherrygramRow;
    private int exteraGramRow;
    private int exteralessRow;
    private int projectsShadowRow;
    private int communitiesHeaderRow;
    private int cherrygramChannelRow;
    private int cherrygramSupportRow;
    private int exteraGramChannelRow;
    private int exteraGramChatRow;
    private int exteralessChannelRow;
    private int communitiesShadowRow;
    private int designHeaderRow;
    private int designRow;
    private int studioRow;
    private int iconAuthorsRow;
    private int designShadowRow;
    private int aboutHeaderRow;
    private int aboutRow;
    private int aboutShadowRow;
    private int licenseHeaderRow;
    private int licenseRow;
    private int licenseShadowRow;

    @Override
    protected void updateRows() {
        super.updateRows();
        projectsHeaderRow = addRow();
        cherrygramRow = addRow();
        exteraGramRow = addRow();
        exteralessRow = addRow();
        projectsShadowRow = addRow();
        communitiesHeaderRow = addRow();
        cherrygramChannelRow = addRow();
        cherrygramSupportRow = addRow();
        exteraGramChannelRow = addRow();
        exteraGramChatRow = addRow();
        exteralessChannelRow = addRow();
        communitiesShadowRow = addRow();
        designHeaderRow = addRow();
        designRow = addRow();
        studioRow = addRow();
        iconAuthorsRow = addRow();
        designShadowRow = addRow();
        aboutHeaderRow = addRow();
        aboutRow = addRow();
        aboutShadowRow = addRow();
        licenseHeaderRow = addRow();
        licenseRow = addRow();
        licenseShadowRow = addRow();
    }

    @Override
    protected String getActionBarTitle() {
        return "Авторы и благодарности";
    }

    @Override
    protected void onItemClick(View view, int position, float x, float y) {
        String url = null;
        if (position == cherrygramRow) {
            url = CHERRYGRAM;
        } else if (position == exteraGramRow) {
            url = EXTERAGRAM;
        } else if (position == exteralessRow) {
            url = EXTERALESS;
        } else if (position == cherrygramChannelRow) {
            url = "https://t.me/cherrygram";
        } else if (position == cherrygramSupportRow) {
            url = "https://t.me/CherrygramSupport";
        } else if (position == exteraGramChannelRow) {
            url = "https://t.me/exteraGram";
        } else if (position == exteraGramChatRow) {
            url = "https://t.me/exteraChat";
        } else if (position == exteralessChannelRow) {
            url = "https://t.me/exteraless";
        } else if (position == designRow) {
            url = "https://t.me/the8055u";
        } else if (position == studioRow) {
            url = "https://t.me/BlueprintDsgn";
        }
        if (url != null) {
            Browser.openUrl(getParentActivity(), url);
        }
    }

    @Override
    protected BaseListAdapter createAdapter(Context context) {
        return new ListAdapter(context);
    }

    private class ListAdapter extends BaseListAdapter {

        ListAdapter(Context context) {
            super(context);
        }

        @Override
        public void onBindViewHolder(@NonNull RecyclerView.ViewHolder holder, int position, boolean partial) {
            switch (holder.getItemViewType()) {
                case TYPE_SHADOW:
                    holder.itemView.setBackground(Theme.getThemedDrawable(mContext,
                            R.drawable.greydivider, Theme.key_windowBackgroundGrayShadow));
                    break;
                case TYPE_HEADER: {
                    HeaderCell headerCell = (HeaderCell) holder.itemView;
                    if (position == projectsHeaderRow) {
                        headerCell.setText("Проекты");
                    } else if (position == communitiesHeaderRow) {
                        headerCell.setText("Сообщества");
                    } else if (position == designHeaderRow) {
                        headerCell.setText("Дизайн");
                    } else if (position == aboutHeaderRow) {
                        headerCell.setText("О grapeGram");
                    } else if (position == licenseHeaderRow) {
                        headerCell.setText("Лицензия");
                    }
                    break;
                }
                case TYPE_INFO_PRIVACY: {
                    TextInfoPrivacyCell cell = (TextInfoPrivacyCell) holder.itemView;
                    cell.setBackground(Theme.getThemedDrawable(mContext,
                            R.drawable.greydivider, Theme.key_windowBackgroundGrayShadow));
                    if (position == iconAuthorsRow) {
                        cell.setText("Авторы community-иконок exteraGram по upstream-каталогу: "
                                + "whatwherewhywhen, cIick_cIack, impulz3, nekto_nkt, MilesBlooper, "
                                + "nneeuurr, n1ksly, weizoqq, ataeshi и Slucket. "
                                + "Официальные варианты указаны за exteraGram.");
                    } else if (position == aboutRow) {
                        cell.setText("grapeGram — независимый Android-клиент на основе закреплённой версии exteraless. "
                                + "Проект объединяет расширенные настройки, плагины, AyuMoments, AI и систему "
                                + "подписанных пользовательских бейджей. Это неофициальный проект, не связанный "
                                + "с Telegram и авторами перечисленных upstream-клиентов.");
                    } else if (position == licenseRow) {
                        cell.setText("Лицензии проектов: grapeGram, exteraGram и exteraless — GNU GPL-3.0; "
                                + "Cherrygram — GNU GPL-2.0. "
                                + "Иконки и компоненты сохраняют условия и авторские сведения исходных проектов. "
                                + "Upstream-проекты не поддерживают grapeGram и не связаны с ним.");
                    }
                    break;
                }
                case TYPE_SETTINGS: {
                    TextSettingsCell cell = (TextSettingsCell) holder.itemView;
                    if (position == cherrygramRow) {
                        cell.setTextAndValue("Cherrygram", "бейджи и часть иконок · GitHub", true);
                    } else if (position == exteraGramRow) {
                        cell.setTextAndValue("exteraGram", "плагины и настройки · GitHub", true);
                    } else if (position == exteralessRow) {
                        cell.setTextAndValue("exteraless", "закреплённая база клиента · GitHub", true);
                    } else if (position == cherrygramChannelRow) {
                        cell.setTextAndValue("Канал Cherrygram", "@cherrygram", true);
                    } else if (position == cherrygramSupportRow) {
                        cell.setTextAndValue("Поддержка Cherrygram", "@CherrygramSupport", true);
                    } else if (position == exteraGramChannelRow) {
                        cell.setTextAndValue("Канал exteraGram", "@exteraGram", true);
                    } else if (position == exteraGramChatRow) {
                        cell.setTextAndValue("Чат exteraGram", "@exteraChat", true);
                    } else if (position == exteralessChannelRow) {
                        cell.setTextAndValue("Канал exteraless", "@exteraless", true);
                    } else if (position == designRow) {
                        cell.setTextAndValue("Иконки и дизайн", "@the8055u", true);
                    } else if (position == studioRow) {
                        cell.setTextAndValue("Студия дизайна", "@BlueprintDsgn", false);
                    }
                    break;
                }
                default:
                    break;
            }
        }

        @Override
        public int getItemViewType(int position) {
            if (position == projectsHeaderRow || position == communitiesHeaderRow
                    || position == designHeaderRow || position == aboutHeaderRow || position == licenseHeaderRow) {
                return TYPE_HEADER;
            }
            if (position == projectsShadowRow || position == communitiesShadowRow
                    || position == designShadowRow || position == aboutShadowRow || position == licenseShadowRow) {
                return TYPE_SHADOW;
            }
            if (position == iconAuthorsRow || position == aboutRow || position == licenseRow) {
                return TYPE_INFO_PRIVACY;
            }
            return TYPE_SETTINGS;
        }
    }
}
